use crate::utils;
use crate::utils::{cratename, is_near_bindgen_wrapped_or_marshall};
use darling::FromDeriveInput;
use proc_macro::{self, TokenStream};
use quote::quote;
use syn::{parse, parse_macro_input, DeriveInput, ItemFn};

#[derive(FromDeriveInput, Default)]
#[darling(default, attributes(ownable), forward_attrs(allow, doc, cfg))]
struct Opts {
    owner_storage_key: Option<String>,
}

pub fn derive_ownable(input: TokenStream) -> TokenStream {
    let cratename = cratename();

    let input = parse_macro_input!(input);
    let opts = Opts::from_derive_input(&input).expect("Wrong options");
    let DeriveInput { ident, .. } = input;

    let owner_storage_key = opts
        .owner_storage_key
        .unwrap_or_else(|| "__OWNER__".to_string());

    let output = quote! {
        #[near_bindgen]
        impl Ownable for #ident {
            fn owner_storage_key(&self) -> Vec<u8> {
                (#owner_storage_key).as_bytes().to_vec()
            }

            fn owner_get(&self) -> Option<::near_sdk::AccountId> {
                ::near_sdk::env::storage_read(&self.owner_storage_key()).map(|owner_bytes| {
                    let owner_raw =
                        String::from_utf8(owner_bytes).expect("Ownable: Invalid string format");
                    std::convert::TryInto::try_into(owner_raw).expect("Ownable: Invalid account id")
                })
            }

            fn owner_set(&mut self, owner: Option<::near_sdk::AccountId>) {
                let current_owner = self.owner_get();

                if let Some(owner) = current_owner.as_ref() {
                    assert_eq!(
                        &::near_sdk::env::predecessor_account_id(),
                        owner,
                        "Ownable: Only owner can update current owner"
                    );
                } else {
                    // If owner is not set, only self can update the owner.
                    // Used mostly on constructor.
                    assert_eq!(
                        ::near_sdk::env::predecessor_account_id(),
                        ::near_sdk::env::current_account_id(),
                        "Ownable: Owner not set. Only self can set the owner"
                    );
                }

                ::near_sdk::log!(#cratename::events::AsEvent::event(
                    &#cratename::ownable::OwnershipTransferred {
                        previous_owner: current_owner,
                        new_owner: owner.clone()
                    }
                ));

                match owner.as_ref() {
                    Some(owner) => ::near_sdk::env::storage_write(
                        &self.owner_storage_key(),
                        owner.as_ref().as_bytes(),
                    ),
                    None => ::near_sdk::env::storage_remove(&self.owner_storage_key()),
                };
            }

            fn owner_is(&self) -> bool {
                self.owner_get().map_or(false, |owner| {
                    owner == ::near_sdk::env::predecessor_account_id()
                })
            }
        }
    };

    output.into()
}

pub fn only(attrs: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse::<ItemFn>(item.clone()).unwrap();
    if is_near_bindgen_wrapped_or_marshall(&input) {
        return item;
    }
    let mut contains_self = false;
    let mut contains_owner = false;
    // TODO: Use darling for this
    for attr in attrs {
        match attr.to_string().as_str() {
            "self" => contains_self = true,
            "owner" => contains_owner = true,
            _ => {}
        }
    }

    let owner_check = match (contains_self, contains_owner) {
        (true, true) => quote! {
            if !self.owner_is() {
                ::near_sdk::assert_self();
            }
        },
        (true, false) => quote! {
            ::near_sdk::assert_self();
        },
        (false, true) => quote! {
            assert!(self.owner_is(), "Ownable: Method must be called from owner");
        },
        (false, false) => {
            panic!("Ownable::only macro target not specified. Select at least one in [self, owner]")
        }
    };

    utils::add_extra_code_to_fn(&input, owner_check)
}
