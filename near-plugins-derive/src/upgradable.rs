use crate::utils::cratename;
use darling::FromDeriveInput;
use proc_macro::{self, TokenStream};
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[derive(FromDeriveInput, Default)]
#[darling(default, attributes(upgradable), forward_attrs(allow, doc, cfg))]
struct Opts {
    storage_prefix: Option<String>,
}

const DEFAULT_STORAGE_PREFIX: &str = "__up__";

pub fn derive_upgradable(input: TokenStream) -> TokenStream {
    let cratename = cratename();

    let input = parse_macro_input!(input);
    let opts = Opts::from_derive_input(&input).expect("Wrong options");
    let DeriveInput { ident, .. } = input;

    let storage_prefix = opts
        .storage_prefix
        .unwrap_or_else(|| DEFAULT_STORAGE_PREFIX.to_string());

    let output = quote! {
        /// Used to make storage prefixes unique. Not to be used directly,
        /// instead it should be prepended to the storage prefix specified by
        /// the user.
        #[derive(::near_sdk::borsh::BorshSerialize)]
        enum __UpgradableStorageKey {
            Code,
            StagingTimestamp,
            StagingDuration,
            UpdateStagingDuration,
            UpdateStagingDurationTimestamp,
        }

        impl #ident {
            fn up_get_timestamp(&self, key: __UpgradableStorageKey) -> Option<::near_sdk::Timestamp> {
                near_sdk::env::storage_read(self.up_storage_key(key).as_ref()).map(|staging_timestamp_bytes| {
                    u64::from_be_bytes(staging_timestamp_bytes.try_into().unwrap_or_else(|_|
                        near_sdk::env::panic_str("Upgradable: Invalid u64 timestamp format"))
                    )
                })
            }

            fn up_get_duration(&self, key: __UpgradableStorageKey) -> Option<::near_sdk::Duration> {
                near_sdk::env::storage_read(self.up_storage_key(key).as_ref()).map(|staging_duration_bytes| {
                    u64::from_be_bytes(staging_duration_bytes.try_into().unwrap_or_else(|_|
                        near_sdk::env::panic_str("Upgradable: Invalid u64 Duration format"))
                    )
                })
            }

            fn up_storage_key(&self, key: __UpgradableStorageKey) -> Vec<u8> {
                let key_vec = key
                    .try_to_vec()
                    .unwrap_or_else(|_| ::near_sdk::env::panic_str("Storage key should be serializable"));
                [(#storage_prefix).as_bytes(), key_vec.as_slice()].concat()
            }
        }

        #[near_bindgen]
        impl Upgradable for #ident {
            fn up_storage_prefix(&self) -> &'static [u8] {
                (#storage_prefix).as_bytes()
            }

            fn up_get_duration_status(&self) -> #cratename::UpgradableDurationStatus {
                near_plugins::UpgradableDurationStatus {
                    staging_duration: self.up_get_duration(__UpgradableStorageKey::StagingDuration),
                    staging_timestamp: self.up_get_timestamp(__UpgradableStorageKey::StagingTimestamp),
                    update_staging_duration: self.up_get_duration(__UpgradableStorageKey::UpdateStagingDuration),
                    update_staging_duration_timestamp: self.up_get_timestamp(__UpgradableStorageKey::UpdateStagingDurationTimestamp),
                }
            }

            #[#cratename::only(owner)]
            fn up_stage_code(&mut self, #[serializer(borsh)] code: Vec<u8>) {
                let timestamp = near_sdk::env::block_timestamp() + self.up_get_duration(__UpgradableStorageKey::StagingDuration).unwrap_or(0);

                if code.is_empty() {
                    near_sdk::env::storage_remove(self.up_storage_key(__UpgradableStorageKey::Code).as_ref());
                } else {
                    near_sdk::env::storage_write(self.up_storage_key(__UpgradableStorageKey::Code).as_ref(), code.as_ref());
                }

                near_sdk::env::storage_write(self.up_storage_key(__UpgradableStorageKey::StagingTimestamp).as_ref(), &timestamp.to_be_bytes());
            }

            #[result_serializer(borsh)]
            fn up_staged_code(&self) -> Option<Vec<u8>> {
                near_sdk::env::storage_read(self.up_storage_key(__UpgradableStorageKey::Code).as_ref())
            }

            fn up_staged_code_hash(&self) -> Option<::near_sdk::CryptoHash> {
                self.up_staged_code()
                    .map(|code| std::convert::TryInto::try_into(near_sdk::env::sha256(code.as_ref())).unwrap())
            }

            #[#cratename::only(owner)]
            fn up_deploy_code(&mut self) -> near_sdk::Promise {
                let staging_timestamp = self.up_get_timestamp(__UpgradableStorageKey::StagingTimestamp).unwrap_or(0);
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

            #[#cratename::only(owner)]
            fn up_init_staging_duration(&self, staging_duration: near_sdk::Duration) {
                near_sdk::require!(self.up_get_duration(__UpgradableStorageKey::StagingDuration).is_none(), "Upgradable: staging duration was already initialized");
                near_sdk::env::storage_write(self.up_storage_key(__UpgradableStorageKey::StagingDuration).as_ref(), &staging_duration.to_be_bytes());
            }

            #[#cratename::only(owner)]
            fn up_stage_update_staging_duration(&self, staging_duration: near_sdk::Duration) {
                let staging_duration_timestamp = near_sdk::env::block_timestamp() + self.up_get_duration(__UpgradableStorageKey::StagingDuration).unwrap_or(0);
                near_sdk::env::storage_write(self.up_storage_key(__UpgradableStorageKey::UpdateStagingDuration).as_ref(), &staging_duration.to_be_bytes());
                near_sdk::env::storage_write(self.up_storage_key(__UpgradableStorageKey::UpdateStagingDurationTimestamp).as_ref(), &staging_duration_timestamp.to_be_bytes());
            }

            #[#cratename::only(owner)]
            fn up_apply_update_staging_duration(&self) {
                let staging_timestamp = self.up_get_timestamp(__UpgradableStorageKey::UpdateStagingDurationTimestamp)
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

                let new_duration = self.up_get_duration(__UpgradableStorageKey::UpdateStagingDuration)
                    .unwrap_or_else(|| ::near_sdk::env::panic_str("Upgradable: No staged duration update"));

                near_sdk::env::storage_write(self.up_storage_key(__UpgradableStorageKey::StagingDuration).as_ref(), &new_duration.to_be_bytes());
            }
        }
    };

    output.into()
}
