use crate::utils::cratename;
use darling::FromDeriveInput;
use proc_macro::{self, TokenStream};
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[derive(FromDeriveInput, Default)]
#[darling(default, attributes(upgradable), forward_attrs(allow, doc, cfg))]
struct Opts {
    code_storage_key: Option<String>,
    staging_timestamp_storage_key: Option<String>,
    staging_duration_storage_key: Option<String>,
    update_staging_duration_storage_key: Option<String>,
    update_staging_duration_timestamp_storage_key: Option<String>,
}

pub fn derive_upgradable(input: TokenStream) -> TokenStream {
    let cratename = cratename();

    let input = parse_macro_input!(input);
    let opts = Opts::from_derive_input(&input).expect("Wrong options");
    let DeriveInput { ident, .. } = input;

    let code_storage_key = opts
        .code_storage_key
        .unwrap_or_else(|| "__CODE__".to_string());

    let staging_timestamp_storage_key = opts
        .staging_timestamp_storage_key
        .unwrap_or_else(|| "__TIMESTAMP__".to_string());

    let staging_duration_storage_key = opts
        .staging_duration_storage_key
        .unwrap_or_else(|| "__DURATION__".to_string());

    let update_staging_duration_storage_key = opts
        .update_staging_duration_storage_key
        .unwrap_or_else(|| "__UPDATE_DURATION__".to_string());

    let update_staging_duration_timestamp_storage_key = opts
        .update_staging_duration_timestamp_storage_key
        .unwrap_or_else(|| "__UPDATE_DURATION_TIMESTAMP__".to_string());

    let output = quote! {
        #[near_bindgen]
        impl Upgradable for #ident {
            fn up_storage_key(&self) -> Vec<u8> {
                (#code_storage_key).as_bytes().to_vec()
            }

            fn up_staging_timestamp_storage_key(&self) -> Vec<u8> {
                (#staging_timestamp_storage_key).as_bytes().to_vec()
            }

            fn up_staging_duration_storage_key(&self) -> Vec<u8> {
                (#staging_duration_storage_key).as_bytes().to_vec()
            }

            fn up_update_staging_duration_storage_key(&self) -> Vec<u8> {
                (#update_staging_duration_storage_key).as_bytes().to_vec()
            }

            fn up_update_staging_duration_timestamp_storage_key(&self) -> Vec<u8> {
                (#update_staging_duration_timestamp_storage_key).as_bytes().to_vec()
            }

            #[#cratename::only(owner)]
            fn up_stage_code(&mut self, #[serializer(borsh)] code: Vec<u8>) {
                let timestamp = near_sdk::env::block_timestamp() + self.up_get_staging_duration().unwrap_or(0);

                if code.is_empty() {
                    near_sdk::env::storage_remove(self.up_storage_key().as_ref());
                } else {
                    near_sdk::env::storage_write(self.up_storage_key().as_ref(), code.as_ref());
                }

                near_sdk::env::storage_write(self.up_staging_timestamp_storage_key().as_ref(), &timestamp.to_be_bytes());
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
                let staging_timestamp = self.up_get_staging_timestamp().unwrap_or(0);
                if staging_timestamp < near_sdk::env::block_timestamp() {
                    near_sdk::env::panic_str(
                        format!(
                            "Upgradable: Deploy code too early: staging ends on {}",
                            staging_timestamp
                        )
                        .as_str(),
                    );
                }

                near_sdk::Promise::new(near_sdk::env::current_account_id())
                    .deploy_contract(self.up_staged_code().unwrap_or_else(|| ::near_sdk::env::panic_str("Upgradable: No staged code")))
            }

            fn up_get_staging_timestamp(&self) -> Option<near_sdk::Timestamp> {
                near_sdk::env::storage_read(self.up_staging_timestamp_storage_key().as_ref()).map(|staging_timestamp_bytes| {
                    u64::from_be_bytes(staging_timestamp_bytes.try_into().unwrap_or_else(|_|
                        near_sdk::env::panic_str("Upgradable: Invalid u64 timestamp format"))
                    )
                })
            }

            fn up_get_staging_duration(&self) -> Option<near_sdk::Duration> {
                near_sdk::env::storage_read(self.up_staging_duration_storage_key().as_ref()).map(|staging_duration_bytes| {
                    u64::from_be_bytes(staging_duration_bytes.try_into().unwrap_or_else(|_|
                        near_sdk::env::panic_str("Upgradable: Invalid u64 Duration format"))
                    )
                })
            }

            #[#cratename::only(owner)]
            fn up_init_staging_duration(&self, staging_duration: near_sdk::Duration) {
                near_sdk::require!(self.up_get_staging_duration().is_none(), "Upgradable: staging duration was already initialized");
                near_sdk::env::storage_write(self.up_staging_duration_storage_key().as_ref(), &staging_duration.to_be_bytes());
            }

            #[#cratename::only(owner)]
            fn up_stage_update_staging_duration(&self, staging_duration: near_sdk::Duration) {
                let staging_duration_timestamp = near_sdk::env::block_timestamp() + self.up_get_staging_duration().unwrap_or(0);
                near_sdk::env::storage_write(self.up_update_staging_duration_storage_key().as_ref(), &staging_duration.to_be_bytes());
                near_sdk::env::storage_write(self.up_update_staging_duration_timestamp_storage_key().as_ref(), &staging_duration_timestamp.to_be_bytes());
            }

            #[#cratename::only(owner)]
            fn up_apply_update_staging_duration(&self) {
                let staging_timestamp = self.up_get_update_staging_duration_timestamp()
                    .unwrap_or_else(|| ::near_sdk::env::panic_str("Upgradable: No staged update"));

                if staging_timestamp < near_sdk::env::block_timestamp() {
                    near_sdk::env::panic_str(
                        format!(
                            "Upgradable: Update duration too early: staging ends on {}",
                            staging_timestamp
                        )
                        .as_str(),
                    );
                }

                near_sdk::env::storage_write(self.up_staging_duration_storage_key().as_ref(), &staging_timestamp.to_be_bytes());
                near_sdk::env::storage_remove(self.up_update_staging_duration_storage_key().as_ref());
            }

            fn up_get_update_staging_duration_timestamp(&self) -> Option<near_sdk::Timestamp> {
                near_sdk::env::storage_read(self.up_update_staging_duration_timestamp_storage_key().as_ref()).map(|timestamp_bytes| {
                    u64::from_be_bytes(timestamp_bytes.try_into().unwrap_or_else(|_|
                        near_sdk::env::panic_str("Upgradable: Invalid u64 timestamp format"))
                    )
                })
            }

            fn up_get_update_staging_duration(&self) -> Option<near_sdk::Duration> {
                near_sdk::env::storage_read(self.up_update_staging_duration_storage_key().as_ref()).map(|duration_bytes| {
                    u64::from_be_bytes(duration_bytes.try_into().unwrap_or_else(|_|
                        near_sdk::env::panic_str("Upgradable: Invalid u64 Duration format"))
                    )
                })
            }
        }
    };

    output.into()
}
