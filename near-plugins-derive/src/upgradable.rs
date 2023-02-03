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

/// Generates the token stream for the `Upgradable` macro.
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
            NewStagingDuration,
            NewStagingDurationTimestamp,
        }

        impl #ident {
            fn up_get_timestamp(&self, key: __UpgradableStorageKey) -> Option<::near_sdk::Timestamp> {
                near_sdk::env::storage_read(self.up_storage_key(key).as_ref()).map(|timestamp_bytes| {
                    ::near_sdk::Timestamp::try_from_slice(&timestamp_bytes).unwrap_or_else(|_|
                        near_sdk::env::panic_str("Upgradable: Invalid u64 timestamp format")
                    )
                })
            }

            fn up_get_duration(&self, key: __UpgradableStorageKey) -> Option<::near_sdk::Duration> {
                near_sdk::env::storage_read(self.up_storage_key(key).as_ref()).map(|duration_bytes| {
                    ::near_sdk::Duration::try_from_slice(&duration_bytes).unwrap_or_else(|_|
                            near_sdk::env::panic_str("Upgradable: Invalid u64 Duration format")
                    )
                })
            }

            fn up_set_timestamp(&self, key: __UpgradableStorageKey, value: ::near_sdk::Timestamp) {
                self.up_storage_write(key, &value.try_to_vec().unwrap());
            }

            fn up_set_duration(&self, key: __UpgradableStorageKey, value: ::near_sdk::Duration) {
                self.up_storage_write(key, &value.try_to_vec().unwrap());
            }

            fn up_storage_key(&self, key: __UpgradableStorageKey) -> Vec<u8> {
                let key_vec = key
                    .try_to_vec()
                    .unwrap_or_else(|_| ::near_sdk::env::panic_str("Storage key should be serializable"));
                [(#storage_prefix).as_bytes(), key_vec.as_slice()].concat()
            }

            fn up_storage_write(&self, key: __UpgradableStorageKey, value: &[u8]) {
                near_sdk::env::storage_write(self.up_storage_key(key).as_ref(), &value);
            }

            fn up_set_staging_duration_unchecked(&self, staging_duration: near_sdk::Duration) {
                self.up_storage_write(__UpgradableStorageKey::StagingDuration, &staging_duration.try_to_vec().unwrap());
            }
        }

        #[near_bindgen]
        impl Upgradable for #ident {
            fn up_storage_prefix(&self) -> &'static [u8] {
                (#storage_prefix).as_bytes()
            }

            fn up_get_delay_status(&self) -> #cratename::UpgradableDurationStatus {
                near_plugins::UpgradableDurationStatus {
                    staging_duration: self.up_get_duration(__UpgradableStorageKey::StagingDuration),
                    staging_timestamp: self.up_get_timestamp(__UpgradableStorageKey::StagingTimestamp),
                    new_staging_duration: self.up_get_duration(__UpgradableStorageKey::NewStagingDuration),
                    new_staging_duration_timestamp: self.up_get_timestamp(__UpgradableStorageKey::NewStagingDurationTimestamp),
                }
            }

            #[#cratename::only(owner)]
            fn up_stage_code(&mut self, #[serializer(borsh)] code: Vec<u8>) {
                if code.is_empty() {
                    near_sdk::env::storage_remove(self.up_storage_key(__UpgradableStorageKey::Code).as_ref());
                    near_sdk::env::storage_remove(self.up_storage_key(__UpgradableStorageKey::StagingTimestamp).as_ref());
                } else {
                    let timestamp = near_sdk::env::block_timestamp() + self.up_get_duration(__UpgradableStorageKey::StagingDuration).unwrap_or(0);
                    self.up_storage_write(__UpgradableStorageKey::Code, &code);
                    self.up_set_timestamp(__UpgradableStorageKey::StagingTimestamp, timestamp);
                }
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
                let staging_timestamp = self.up_get_timestamp(__UpgradableStorageKey::StagingTimestamp)
                    .unwrap_or_else(|| ::near_sdk::env::panic_str("Upgradable: staging timestamp isn't set"));

                if near_sdk::env::block_timestamp() < staging_timestamp {
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
            fn up_init_staging_duration(&mut self, staging_duration: near_sdk::Duration) {
                near_sdk::require!(self.up_get_duration(__UpgradableStorageKey::StagingDuration).is_none(), "Upgradable: staging duration was already initialized");
                self.up_set_staging_duration_unchecked(staging_duration);
            }

            #[#cratename::only(owner)]
            fn up_stage_update_staging_duration(&mut self, staging_duration: near_sdk::Duration) {
                let current_staging_duration = self.up_get_duration(__UpgradableStorageKey::StagingDuration)
                    .unwrap_or_else(|| ::near_sdk::env::panic_str("Upgradable: staging duration isn't initialized"));

                self.up_set_duration(__UpgradableStorageKey::NewStagingDuration, staging_duration);
                let staging_duration_timestamp = near_sdk::env::block_timestamp() + current_staging_duration;
                self.up_set_timestamp(__UpgradableStorageKey::NewStagingDurationTimestamp, staging_duration_timestamp);
            }

            #[#cratename::only(owner)]
            fn up_apply_update_staging_duration(&mut self) {
                let staging_timestamp = self.up_get_timestamp(__UpgradableStorageKey::NewStagingDurationTimestamp)
                    .unwrap_or_else(|| ::near_sdk::env::panic_str("Upgradable: No staged update"));

                if near_sdk::env::block_timestamp() < staging_timestamp {
                    near_sdk::env::panic_str(
                        format!(
                            "Upgradable: Update duration too early: staging ends on {}",
                            staging_timestamp
                        )
                        .as_str(),
                    );
                }

                let new_duration = self.up_get_duration(__UpgradableStorageKey::NewStagingDuration)
                    .unwrap_or_else(|| ::near_sdk::env::panic_str("Upgradable: No staged duration update"));

                self.up_set_duration(__UpgradableStorageKey::StagingDuration, new_duration);
            }
        }
    };

    output.into()
}
