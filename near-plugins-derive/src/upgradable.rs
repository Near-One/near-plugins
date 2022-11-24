use crate::utils::cratename;
use darling::FromDeriveInput;
use proc_macro::{self, TokenStream};
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[derive(FromDeriveInput, Default)]
#[darling(default, attributes(upgradable), forward_attrs(allow, doc, cfg))]
struct Opts {
    code_storage_key: Option<String>,
    allowed_timestamp_storage_key: Option<String>,
}

pub fn derive_upgradable(input: TokenStream) -> TokenStream {
    let cratename = cratename();

    let input = parse_macro_input!(input);
    let opts = Opts::from_derive_input(&input).expect("Wrong options");
    let DeriveInput { ident, .. } = input;

    let code_storage_key = opts
        .code_storage_key
        .unwrap_or_else(|| "__CODE__".to_string());

    let allowed_timestamp_storage_key = opts
        .allowed_timestamp_storage_key
        .unwrap_or_else(|| "__ALLOWED_TIMESTAMP__".to_string());

    let output = quote! {
        #[near_bindgen]
        impl Upgradable for #ident {
            fn up_storage_key(&self) -> Vec<u8> {
                (#code_storage_key).as_bytes().to_vec()
            }

            fn up_allowed_timestamp_storage_key(&self) -> Vec<u8> {
                (#allowed_timestamp_storage_key).as_bytes().to_vec()
            }

            #[#cratename::only(owner)]
            fn up_stage_code(&mut self, #[serializer(borsh)] code: Vec<u8>,  #[serializer(borsh)] delay_timestamp: u64) {
                if code.is_empty() {
                    near_sdk::env::storage_remove(self.up_storage_key().as_ref());
                    near_sdk::env::storage_remove(self.up_allowed_timestamp_storage_key().as_ref());
                } else {
                    near_sdk::env::storage_write(self.up_storage_key().as_ref(), code.as_ref());
                    near_sdk::env::storage_write(self.up_allowed_timestamp_storage_key().as_ref(), &(near_sdk::env::block_timestamp() + delay_timestamp).to_be_bytes());
                }
            }

            #[result_serializer(borsh)]
            fn up_staged_code(&self) -> Option<Vec<u8>> {
                near_sdk::env::storage_read(self.up_storage_key().as_ref())
            }

            fn up_staged_code_hash(&self) -> Option<::near_sdk::CryptoHash> {
                self.up_staged_code()
                    .map(|code| std::convert::TryInto::try_into(near_sdk::env::sha256(code.as_ref())).unwrap())
            }

            #[#cratename::only(owner)]
            fn up_deploy_code(&mut self) -> near_sdk::Promise {
                near_sdk::require!(self.up_allowed_timestamp().unwrap_or(0) <= near_sdk::env::block_timestamp(), "Upgradable: The delay time hasn't yet passed");

                near_sdk::Promise::new(near_sdk::env::current_account_id())
                    .deploy_contract(self.up_staged_code().unwrap_or_else(|| ::near_sdk::env::panic_str("Upgradable: No staged code")))
            }

            fn up_allowed_timestamp(&self) -> Option<u64> {
                near_sdk::env::storage_read(self.up_allowed_timestamp_storage_key().as_ref()).map(|allowed_timestamp_bytes| {
                    u64::from_be_bytes(allowed_timestamp_bytes.try_into().unwrap_or_else(|_| ::near_sdk::env::panic_str("Upgradable: Invalid u64 timestamp format")))
                })
            }
        }
    };

    output.into()
}
