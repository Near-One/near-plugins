use proc_macro::{self, TokenStream};
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

pub fn derive_fak_fallback(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input);
    let DeriveInput { ident, .. } = input;

    let output = quote! {
        #[near_bindgen]
        impl FullAccessKeyFallback for #ident {
            #[only(owner)]
            fn attach_full_access_key(&mut self, public_key: ::near_sdk::PublicKey) -> near_sdk::Promise {
                ::near_sdk::Promise::new(::near_sdk::env::current_account_id())
                    .add_full_access_key(public_key)
            }
        }
    };

    output.into()
}
