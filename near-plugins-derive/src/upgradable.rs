use darling::FromDeriveInput;
use proc_macro::{self, TokenStream};
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[derive(FromDeriveInput, Default)]
#[darling(default, attributes(upgradable), forward_attrs(allow, doc, cfg))]
struct Opts {
    code_storage_key: Option<String>,
}

pub fn derive_upgradable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input);
    let opts = Opts::from_derive_input(&input).expect("Wrong options");
    let DeriveInput { ident, .. } = input;

    let code_storage_key = opts.code_storage_key.unwrap_or("__CODE__".to_string());

    let output = quote! {
        #[near_bindgen]
        impl Upgradable for #ident {
            fn code_storage_key(&self) -> Vec<u8>{
                (#code_storage_key).as_bytes().to_vec()
            }

            #[check_only(self, owner)]
            fn stage_code(&mut self, #[serializer(borsh)] code: Vec<u8>) {
                if code.is_empty() {
                    near_sdk::env::storage_remove(self.code_storage_key().as_ref());
                } else {
                    near_sdk::env::storage_write(self.code_storage_key().as_ref(), code.as_ref());
                }
            }

            #[result_serializer(borsh)]
            fn get_staged_code(&self) -> Option<Vec<u8>> {
                near_sdk::env::storage_read(self.code_storage_key().as_ref())
            }

            fn get_staged_code_hash(&self) -> Option<String> {
                self.get_staged_code()
                    .map(|code| near_sdk::env::sha256(code.as_ref()))
                    .map(|hash| near_sdk::bs58::encode(hash).into_string())
            }

            #[check_only(self, owner)]
            fn deploy_code(&mut self) -> near_sdk::Promise {
                near_sdk::Promise::new(near_sdk::env::current_account_id())
                    .deploy_contract(self.get_staged_code().expect("Upgradable: No staged code"))
            }
        }
    };

    output.into()
}
