use crate::utils;
use crate::utils::{cratename, is_near_bindgen_wrapped_or_marshall};
use darling::util::PathList;
use darling::{FromDeriveInput, FromMeta};
use proc_macro::{self, TokenStream};
use quote::quote;
use syn::{parse, parse_macro_input, AttributeArgs, DeriveInput, ItemFn};

#[derive(FromDeriveInput, Default)]
#[darling(default, attributes(pausable), forward_attrs(allow, doc, cfg))]
struct Opts {
    /// Storage key under which the set of paused features is stored. If it is
    /// `None` the default value will be used.
    paused_storage_key: Option<String>,
    /// Access control roles whose grantees may pause and unpause features.
    manager_roles: PathList,
}

/// Generates the token stream that implements `Pausable`.
pub fn derive_pausable(input: TokenStream) -> TokenStream {
    let cratename = cratename();

    let input = parse_macro_input!(input);
    let opts = Opts::from_derive_input(&input).expect("Wrong options");
    let DeriveInput { ident, .. } = input;

    let paused_storage_key = opts
        .paused_storage_key
        .unwrap_or_else(|| "__PAUSE__".to_string());
    let manager_roles = opts.manager_roles;
    assert!(
        manager_roles.len() > 0,
        "Specify at least one role for manager_roles"
    );

    let output = quote! {
        #[near]
        impl #cratename::Pausable for #ident {
            fn pa_storage_key(&self) -> &'static [u8] {
                (#paused_storage_key).as_bytes()
            }

            fn pa_is_paused(&self, key: String) -> bool {
                self.pa_all_paused()
                    .map(|keys| keys.contains(&key) || keys.contains("ALL"))
                    .unwrap_or(false)
            }

            fn pa_all_paused(&self) -> Option<std::collections::HashSet<String>> {
                near_sdk::env::storage_read(self.pa_storage_key().as_ref()).map(|value| {
                    std::collections::HashSet::try_from_slice(value.as_ref())
                        .unwrap_or_else(|_| near_sdk::env::panic_str("Pausable: Invalid format for paused keys"))
                })
            }

            #[#cratename::access_control_any(roles(#(#manager_roles),*))]
            fn pa_pause_feature(&mut self, key: String) -> bool {
                let mut paused_keys = self.pa_all_paused().unwrap_or_default();
                let newly_paused = paused_keys.insert(key.clone());

                if !newly_paused {
                    // Nothing to do since state was not modified.
                    return false;
                }

                near_sdk::env::storage_write(
                    self.pa_storage_key().as_ref(),
                    near_sdk::borsh::to_vec(&paused_keys)
                        .unwrap_or_else(|_| near_sdk::env::panic_str("Pausable: Unexpected error serializing keys"))
                        .as_ref(),
                );

                let event = #cratename::pausable::Pause {
                    by: near_sdk::env::predecessor_account_id(),
                    key,
                };
                #cratename::events::AsEvent::emit(&event);

                // The feature is newly paused.
                true
            }

            #[#cratename::access_control_any(roles(#(#manager_roles),*))]
            fn pa_unpause_feature(&mut self, key: String) -> bool {
                let mut paused_keys = self.pa_all_paused().unwrap_or_default();
                let was_paused = paused_keys.remove(&key);

                if !was_paused {
                    // Nothing to do since state is not modified.
                    return false;
                }

                if paused_keys.is_empty() {
                    near_sdk::env::storage_remove(self.pa_storage_key().as_ref());
                } else {
                    near_sdk::env::storage_write(
                        self.pa_storage_key().as_ref(),
                        near_sdk::borsh::to_vec(&paused_keys)
                            .unwrap_or_else(|_| near_sdk::env::panic_str("Pausable: Unexpected error serializing keys"))
                            .as_ref(),
                    );
                }

                let event = #cratename::pausable::Unpause {
                    by: near_sdk::env::predecessor_account_id(),
                    key,
                };
                #cratename::events::AsEvent::emit(&event);

                // The feature was paused.
                true
            }
        }
    };

    output.into()
}

/// Defines sub-attributes for the `except` attribute.
#[derive(Default, FromMeta, Debug)]
#[darling(default)]
pub struct ExceptSubArgs {
    /// Grantees of these roles are exempted and may always call the method.
    roles: PathList,
}

/// Defines attributes for the `pause` macro.
#[derive(Debug, FromMeta)]
pub struct PauseArgs {
    #[darling(default)]
    name: Option<String>,
    #[darling(default)]
    except: ExceptSubArgs,
}

/// Generates the token stream for the `pause` macro.
pub fn pause(attrs: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse::<ItemFn>(item.clone()).unwrap();

    if is_near_bindgen_wrapped_or_marshall(&input) {
        return item;
    }

    let attr_args = parse_macro_input!(attrs as AttributeArgs);
    let args = PauseArgs::from_list(&attr_args).expect("Invalid arguments");

    let fn_name = args.name.unwrap_or_else(|| input.sig.ident.to_string());

    let bypass_condition = get_bypass_condition(&args.except);

    let check_pause = quote!(
        let mut __check_paused = true;
        #bypass_condition
        if __check_paused {
            near_sdk::require!(!self.pa_is_paused(#fn_name.to_string()), "Pausable: Method is paused");
        }
    );

    utils::add_extra_code_to_fn(&input, &check_pause)
}

/// Defines attributes for the `if_paused` macro.
#[derive(Debug, FromMeta)]
pub struct IfPausedArgs {
    name: String,
    #[darling(default)]
    except: ExceptSubArgs,
}

/// Generates the token stream for the `if_paused` macro.
pub fn if_paused(attrs: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse::<ItemFn>(item.clone()).unwrap();

    if is_near_bindgen_wrapped_or_marshall(&input) {
        return item;
    }

    let attr_args = parse_macro_input!(attrs as AttributeArgs);
    let args = IfPausedArgs::from_list(&attr_args).expect("Invalid arguments");

    let fn_name = args.name;

    // Construct error messages that use `format!` here, i.e. at compile time. Doing that during
    // contract execution would cost extra gas.
    let err_feature_not_paused = format!("Pausable: {fn_name} must be paused to use this function");

    let bypass_condition = get_bypass_condition(&args.except);

    let check_pause = quote!(
        let mut __check_paused = true;
        #bypass_condition
        if __check_paused {
            near_sdk::require!(
                self.pa_is_paused(#fn_name.to_string()),
                #err_feature_not_paused,
            );
        }
    );

    utils::add_extra_code_to_fn(&input, &check_pause)
}

fn get_bypass_condition(args: &ExceptSubArgs) -> proc_macro2::TokenStream {
    let except_roles = args.roles.clone();
    quote!(
        let __except_roles: Vec<&str> = vec![#(#except_roles.into()),*];
        let __except_roles: Vec<String> = __except_roles.iter().map(|&x| x.into()).collect();
        let may_bypass = self.acl_has_any_role(
            __except_roles,
            near_sdk::env::predecessor_account_id()
        );
        if may_bypass {
            __check_paused = false;
        }
    )
}
