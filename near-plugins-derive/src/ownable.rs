use crate::utils::is_near_bindgen_wrapped_code;
use darling::FromDeriveInput;
use proc_macro::{self, TokenStream};
use quote::quote;
use syn::{parse, ItemFn};
use syn::{parse_macro_input, DeriveInput};

#[derive(FromDeriveInput, Default)]
#[darling(default, attributes(ownable), forward_attrs(allow, doc, cfg))]
struct Opts {
    owner_storage_key: Option<String>,
}

pub fn derive_ownable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input);
    let opts = Opts::from_derive_input(&input).expect("Wrong options");
    let DeriveInput { ident, .. } = input;

    let owner_storage_key = opts.owner_storage_key.unwrap_or("__OWNER".to_string());

    let output = quote! {
        #[near_bindgen]
        impl Ownable for #ident {
            fn owner_storage_key(&self) -> Vec<u8> {
                (#owner_storage_key).as_bytes().to_vec()
            }
            fn get_owner(&self) -> Option<AccountId> {
                ::near_sdk::env::storage_read(&self.owner_storage_key()).map(|owner_bytes| {
                    let owner_raw = String::from_utf8(owner_bytes).expect("Ownable: Invalid string format");
                    owner_raw.try_into().expect("Ownable: Invalid account id")
                })
            }

            fn set_owner(&mut self, owner: AccountId) {
                self.assert_owner_or_self();
                env::storage_write(&self.owner_storage_key(), owner.as_ref().as_bytes());
            }

            fn is_owner(&self) -> bool {
                self.get_owner().map_or(false, |owner| {
                    owner == env::predecessor_account_id()
                })
            }
        }
    };

    output.into()
}

pub fn check_only(attrs: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse::<ItemFn>(item.clone()).unwrap();
    if is_near_bindgen_wrapped_code(&input) {
        return item;
    }
    let mut contains_self = false;
    let mut contains_owner = false;
    for attr in attrs {
        match attr.to_string().as_str() {
            "self" => contains_self = true,
            "owner" => contains_owner = true,
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

    let owner_check = match (contains_self, contains_owner) {
            (true, true) => quote! {
                self.assert_owner_or_self();
            },
            (true, false) => quote! {
                ::near_sdk::assert_self();
            },
            (false, true) => quote! {
                self.assert_owner();
            },
            (false, false) => panic!("check_only attribute doesn't specify which account to check. Select at least one in [self, owner]."),
        };

    // https://stackoverflow.com/a/66851407
    quote! {
        #(#attrs)* #vis #sig {
            #owner_check
            #(#stmts)*
        }
    }
    .into()
}
