use crate::utils::cratename;
use darling::util::PathList;
use darling::{FromDeriveInput, FromMeta};
use proc_macro::{self, TokenStream};
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[derive(FromDeriveInput, Default)]
#[darling(default, attributes(upgradable), forward_attrs(allow, doc, cfg))]
struct Opts {
    /// Storage prefix under which this plugin stores its state. If it is `None` the default value
    /// will be used.
    storage_prefix: Option<String>,
    /// Roles which are permitted to call protected methods.
    access_control_roles: AccessControlRoles,
}

/// Specifies which `AccessControlRole`s may call protected methods.
///
/// All field names need to be passed to calls of `check_roles_specified_for!`.
#[derive(Default, FromMeta, Debug)]
#[darling(default)]
struct AccessControlRoles {
    /// Grantess of these roles may successfully call `Upgradable::up_stage_code`.
    code_stagers: PathList,
    /// Grantess of these roles may successfully call `Upgradable::up_deploy_code`.
    code_deployers: PathList,
    /// Grantess of these roles may successfully call `Upgradable::up_init_staging_duration`.
    duration_initializers: PathList,
    /// Grantess of these roles may successfully call `Upgradable::up_stage_update_staging_duration`.
    duration_update_stagers: PathList,
    /// Grantess of these roles may successfully call `Upgradable::up_apply_update_staging_duration`.
    duration_update_appliers: PathList,
}

impl AccessControlRoles {
    /// Validates the roles provided by the plugin user and panics if they are invalid.
    fn validate(&self) {
        // Ensure at least one role is provided for every field of `AccessControlRoles`.
        let mut missing_roles = vec![];

        macro_rules! check_roles_specified_for {
            ($($field_name:ident),+) => (
                $(
                if self.$field_name.len() == 0 {
                    missing_roles.push(stringify!($field_name));
                }
                )+
            )
        }

        check_roles_specified_for!(
            code_stagers,
            code_deployers,
            duration_initializers,
            duration_update_stagers,
            duration_update_appliers
        );
        assert!(
            missing_roles.is_empty(),
            "Specify access_control_roles for: {:?}",
            missing_roles,
        );
    }
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
    let acl_roles = opts.access_control_roles;
    acl_roles.validate();

    // To use fields of a struct inside `quote!`, they must be lifted into variables, see
    // https://github.com/dtolnay/quote/pull/88#pullrequestreview-180577592
    let acl_roles_code_stagers = acl_roles.code_stagers;
    let acl_roles_code_deployers = acl_roles.code_deployers;
    let acl_roles_duration_initializers = acl_roles.duration_initializers;
    let acl_roles_duration_update_stagers = acl_roles.duration_update_stagers;
    let acl_roles_duration_update_appliers = acl_roles.duration_update_appliers;

    let output = quote! {
        /// Used to make storage prefixes unique. Not to be used directly,
        /// instead it should be prepended to the storage prefix specified by
        /// the user.
        #[derive(::near_sdk::borsh::BorshSerialize)]
        #[borsh(crate = "near_sdk::borsh")]
        enum __UpgradableStorageKey {
            Code,
            StagingTimestamp,
            StagingDuration,
            NewStagingDuration,
            NewStagingDurationTimestamp,
        }

        impl #ident {
            fn up_get_timestamp(key: __UpgradableStorageKey) -> Option<::near_sdk::Timestamp> {
                near_sdk::env::storage_read(Self::up_storage_key(key).as_ref()).map(|timestamp_bytes| {
                    ::near_sdk::Timestamp::try_from_slice(&timestamp_bytes).unwrap_or_else(|_|
                        near_sdk::env::panic_str("Upgradable: Invalid u64 timestamp format")
                    )
                })
            }

            fn up_get_duration(key: __UpgradableStorageKey) -> Option<::near_sdk::Duration> {
                ::near_sdk::env::storage_read(Self::up_storage_key(key).as_ref()).map(|duration_bytes| {
                    ::near_sdk::Duration::try_from_slice(&duration_bytes).unwrap_or_else(|_|
                            near_sdk::env::panic_str("Upgradable: Invalid u64 Duration format")
                    )
                })
            }

            fn up_set_timestamp(key: __UpgradableStorageKey, value: ::near_sdk::Timestamp) {
                Self::up_storage_write(key, &::near_sdk::borsh::to_vec(&value).unwrap());
            }

            fn up_set_duration(key: __UpgradableStorageKey, value: ::near_sdk::Duration) {
                Self::up_storage_write(key, &::near_sdk::borsh::to_vec(&value).unwrap());
            }

            fn up_storage_key(key: __UpgradableStorageKey) -> Vec<u8> {
                let key_vec = ::near_sdk::borsh::to_vec(&key)
                    .unwrap_or_else(|_| ::near_sdk::env::panic_str("Storage key should be serializable"));
                [(#storage_prefix).as_bytes(), key_vec.as_slice()].concat()
            }

            fn up_storage_write(key: __UpgradableStorageKey, value: &[u8]) {
                ::near_sdk::env::storage_write(Self::up_storage_key(key).as_ref(), &value);
            }

            fn up_set_staging_duration_unchecked(staging_duration: near_sdk::Duration) {
                Self::up_storage_write(__UpgradableStorageKey::StagingDuration, &::near_sdk::borsh::to_vec(&staging_duration).unwrap());
            }

            /// Computes the `sha256` hash of `code` and panics if the conversion to `CryptoHash` fails.
            fn up_hash_code(code: &[u8]) -> ::near_sdk::CryptoHash {
                let hash = near_sdk::env::sha256(code);
                std::convert::TryInto::try_into(hash)
                    .expect("sha256 should convert to CryptoHash")
            }
        }

        #[near]
        impl Upgradable for #ident {
            fn up_storage_prefix() -> &'static [u8] {
                (#storage_prefix).as_bytes()
            }

            fn up_get_delay_status() -> #cratename::UpgradableDurationStatus {
                #cratename::UpgradableDurationStatus {
                    staging_duration: Self::up_get_duration(__UpgradableStorageKey::StagingDuration),
                    staging_timestamp: Self::up_get_timestamp(__UpgradableStorageKey::StagingTimestamp),
                    new_staging_duration: Self::up_get_duration(__UpgradableStorageKey::NewStagingDuration),
                    new_staging_duration_timestamp: Self::up_get_timestamp(__UpgradableStorageKey::NewStagingDurationTimestamp),
                }
            }

            #[#cratename::access_control_any(roles(#(#acl_roles_code_stagers),*))]
            fn up_stage_code(#[serializer(borsh)] code: Vec<u8>) {
                if code.is_empty() {
                    ::near_sdk::env::storage_remove(Self::up_storage_key(__UpgradableStorageKey::Code).as_ref());
                    ::near_sdk::env::storage_remove(Self::up_storage_key(__UpgradableStorageKey::StagingTimestamp).as_ref());
                } else {
                    let timestamp = ::near_sdk::env::block_timestamp() + Self::up_get_duration(__UpgradableStorageKey::StagingDuration).unwrap_or(0);
                    Self::up_storage_write(__UpgradableStorageKey::Code, &code);
                    Self::up_set_timestamp(__UpgradableStorageKey::StagingTimestamp, timestamp);
                }
            }

            #[result_serializer(borsh)]
            fn up_staged_code() -> Option<Vec<u8>> {
                ::near_sdk::env::storage_read(Self::up_storage_key(__UpgradableStorageKey::Code).as_ref())
            }

            fn up_staged_code_hash() -> Option<::near_sdk::CryptoHash> {
                Self::up_staged_code()
                    .map(|code| Self::up_hash_code(code.as_ref()))
            }

            #[#cratename::access_control_any(roles(#(#acl_roles_code_deployers),*))]
            fn up_deploy_code(hash: String, function_call_args: Option<#cratename::upgradable::FunctionCallArgs>) -> near_sdk::Promise {
                let staging_timestamp = Self::up_get_timestamp(__UpgradableStorageKey::StagingTimestamp)
                    .unwrap_or_else(|| ::near_sdk::env::panic_str("Upgradable: staging timestamp isn't set"));

                if ::near_sdk::env::block_timestamp() < staging_timestamp {
                    ::near_sdk::env::panic_str(
                        format!(
                            "Upgradable: Deploy code too early: staging ends on {}",
                            staging_timestamp
                        )
                        .as_str(),
                    );
                }

                let code = Self::up_staged_code().unwrap_or_else(|| ::near_sdk::env::panic_str("Upgradable: No staged code"));
                let expected_hash = ::near_sdk::base64::encode(Self::up_hash_code(code.as_ref()));
                if hash != expected_hash {
                    near_sdk::env::panic_str(
                        format!(
                            "Upgradable: Cannot deploy due to wrong hash: expected hash: {}",
                            expected_hash,
                        )
                        .as_str(),
                    )
                }

                let promise = ::near_sdk::Promise::new(::near_sdk::env::current_account_id())
                    .deploy_contract(code);
                match function_call_args {
                    None => promise,
                    Some(args) => {
                        // Execute the `DeployContract` and `FunctionCall` actions in a batch
                        // transaction to make a failure of the function call roll back the code
                        // deployment.
                        promise.function_call(args.function_name, args.arguments, args.amount, args.gas)
                    },
                }
            }

            #[#cratename::access_control_any(roles(#(#acl_roles_duration_initializers),*))]
            fn up_init_staging_duration(staging_duration: ::near_sdk::Duration) {
                ::near_sdk::require!(Self::up_get_duration(__UpgradableStorageKey::StagingDuration).is_none(), "Upgradable: staging duration was already initialized");
                Self::up_set_staging_duration_unchecked(staging_duration);
            }

            #[#cratename::access_control_any(roles(#(#acl_roles_duration_update_stagers),*))]
            fn up_stage_update_staging_duration(staging_duration: ::near_sdk::Duration) {
                let current_staging_duration = Self::up_get_duration(__UpgradableStorageKey::StagingDuration)
                    .unwrap_or_else(|| ::near_sdk::env::panic_str("Upgradable: staging duration isn't initialized"));

                Self::up_set_duration(__UpgradableStorageKey::NewStagingDuration, staging_duration);
                let staging_duration_timestamp = ::near_sdk::env::block_timestamp() + current_staging_duration;
                Self::up_set_timestamp(__UpgradableStorageKey::NewStagingDurationTimestamp, staging_duration_timestamp);
            }

            #[#cratename::access_control_any(roles(#(#acl_roles_duration_update_appliers),*))]
            fn up_apply_update_staging_duration() {
                let staging_timestamp = Self::up_get_timestamp(__UpgradableStorageKey::NewStagingDurationTimestamp)
                    .unwrap_or_else(|| ::near_sdk::env::panic_str("Upgradable: No staged update"));

                if ::near_sdk::env::block_timestamp() < staging_timestamp {
                    ::near_sdk::env::panic_str(
                        format!(
                            "Upgradable: Update duration too early: staging ends on {}",
                            staging_timestamp
                        )
                        .as_str(),
                    );
                }

                let new_duration = Self::up_get_duration(__UpgradableStorageKey::NewStagingDuration)
                    .unwrap_or_else(|| ::near_sdk::env::panic_str("Upgradable: No staged duration update"));

                Self::up_set_duration(__UpgradableStorageKey::StagingDuration, new_duration);
            }
        }
    };

    output.into()
}
