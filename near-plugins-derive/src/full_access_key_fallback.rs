use crate::utils::cratename;
use proc_macro::{self, TokenStream};
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

/// Generates the token stream that implements `FullAccessKeyFallback`.
pub fn derive_fak_fallback(input: TokenStream) -> TokenStream {
    let cratename = cratename();

    let input = parse_macro_input!(input);
    let DeriveInput { ident, .. } = input;

    let output = quote! {
        #[near_bindgen]
        impl FullAccessKeyFallback for #ident {
            #[#cratename::only(owner)]
            fn attach_full_access_key(&mut self, public_key: ::near_sdk::PublicKey) -> near_sdk::Promise {
                let current_account_id = ::near_sdk::env::current_account_id();
                // TODO: Use .emit
                ::near_sdk::log!(#cratename::events::AsEvent::event(
                    &#cratename::full_access_key_fallback::FullAccessKeyAdded {
                        by: current_account_id.clone(),
                        public_key: public_key.clone()
                    }
                ));
                ::near_sdk::Promise::new(current_account_id).add_full_access_key(public_key)
            }
        }
    };

    output.into()
}
