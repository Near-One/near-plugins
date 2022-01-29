use crate::utils::is_near_bindgen_wrapped_or_marshall;
use darling::FromDeriveInput;
use proc_macro::{self, TokenStream};
use quote::quote;
use syn::{parse, parse_macro_input, DeriveInput, ItemFn};

#[derive(FromDeriveInput, Default)]
#[darling(default, attributes(pausable), forward_attrs(allow, doc, cfg))]
struct Opts {
    paused_storage_key: Option<String>,
}

pub fn derive_pausable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input);
    let opts = Opts::from_derive_input(&input).expect("Wrong options");
    let DeriveInput { ident, .. } = input;

    let paused_storage_key = opts.paused_storage_key.unwrap_or("__PAUSE".to_string());

    let output = quote! {
        #[near_bindgen]
        impl Pausable for #ident {
            fn paused_storage_key(&self) -> Vec<u8>{
                (#paused_storage_key).as_bytes().to_vec()
            }

            fn is_paused(&self, key: String) -> bool {
                self.paused_keys()
                    .map(|keys| keys.contains(&key) || keys.contains("ALL"))
                    .unwrap_or(false)
            }

            fn paused_keys(&self) -> Option<std::collections::HashSet<String>> {
                near_sdk::env::storage_read(self.paused_storage_key().as_ref()).map(|value| {
                    std::collections::HashSet::try_from_slice(value.as_ref()).expect("Pausable: Invalid format for paused keys")
                })
            }

            #[check_only(self, owner)]
            fn pause(&mut self, key: String) {
                let mut paused_keys = self.paused_keys().unwrap_or_default();
                paused_keys.insert(key);
                near_sdk::env::storage_write(
                    self.paused_storage_key().as_ref(),
                    paused_keys
                        .try_to_vec()
                        .expect("Pausable: Unexpected error serializing keys")
                        .as_ref(),
                );
            }

            #[check_only(self, owner)]
            fn unpause(&mut self, key: String) {
                let mut paused_keys = self.paused_keys().unwrap_or_default();
                paused_keys.remove(&key);

                if paused_keys.is_empty() {
                    near_sdk::env::storage_remove(self.paused_storage_key().as_ref());
                } else {
                    near_sdk::env::storage_write(
                        self.paused_storage_key().as_ref(),
                        paused_keys
                            .try_to_vec()
                            .expect("Pausable: Unexpected error serializing keys")
                            .as_ref(),
                    );
                }
            }
        }
    };

    output.into()
}

pub fn pause(attrs: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse::<ItemFn>(item.clone()).unwrap();

    if is_near_bindgen_wrapped_or_marshall(&input) {
        return item;
    }

    let mut allow_admin = false;
    for attr in attrs {
        match attr.to_string().as_str() {
            "allow_admin" => allow_admin = true,
            _ => {}
        }
    }

    let ItemFn {
        attrs,
        vis,
        sig,
        block,
    } = input;
    let stmts = &block.stmts;

    let fn_name = sig.ident.clone();

    let check_pause = if allow_admin {
        quote!(if !(self.is_owner() || self.is_self()) {
            assert!(!self.is_paused(stringify!(#fn_name).to_string()));
        })
    } else {
        quote!(
            assert!(!self.is_paused(stringify!(#fn_name).to_string()));
        )
    };

    // https://stackoverflow.com/a/66851407
    quote! {
        #(#attrs)* #vis #sig {
            #check_pause
            #(#stmts)*
        }
    }
    .into()
}

// TODO: Macro that only runs when some functionality is paused #[if_paused("features")]
