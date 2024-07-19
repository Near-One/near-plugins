//! # Upgradable
//!
//! Upgradable trait inspired by [NEP123](https://github.com/near/NEPs/pull/123).
//!
//! Using the `Upgradable` plugin requires a contract to be `AccessControllable`.
//!
//! To upgrade the contract, first the code needs to be staged via [`Upgradable::up_stage_code`].
//! Staged code can then be deployed by calling [`Upgradable::up_deploy_code`]. Optionally a staging
//! duration can be set, which defines the minimum duration that must pass before staged code can be
//! deployed.
//!
//! The staging duration defaults to zero, allowing staged code to be deployed immediately. To set a
//! staging duration, call [`Upgradable::up_init_staging_duration`]. After initialization the
//! staging duration can be updated by calling [`Upgradable::up_stage_update_staging_duration`]
//! followed by [`Upgradable::up_apply_update_staging_duration`]. Updating the staging duration is
//! itself subject to a delay: at least the currently set staging duration must pass before a staged
//! update can be applied.
//!
//! ## Permissions
//!
//! The `Upgradable` methods mentioned above are protected by `AccessControllable`. Only accounts
//! that have been granted one of the whitelisted roles may successfully call the corresponding
//! method. The documentation of these methods and the [example contract] explain how to define and
//! whitelist roles to manage authorization for the `Upgradable` plugin.
//!
//! ## State migration
//!
//! Upgrading a contract might require [state migration]. The `Upgradable` plugin allows to attach a
//! function call to code deployments. Using this mechanism, state migration can be carried out by
//! calling a migration function. If the function fails, the deployment is rolled back and the
//! initial code remains active. More detailed information is available in the documentation of
//! [`Upgradable::up_deploy_code`].
//!
//! ## Stale staged code
//!
//! After the code is deployed, it should be removed from staging to unstake tokens and avoid the
//! issues described in [`Self::up_deploy_code`].
//!
//! ## Upgrading code that contains a security vulnerability
//!
//! Once code is staged for an upgrade, it is publicly visible via [`Upgradable::up_staged_code`].
//! Staged code that fixes a security vulnerability might be discovered by an attacker who then
//! exploits the vulnerability before its fix is deployed.
//!
//! To avoid that, the upgrade can be executed by calling [`Upgradable::up_stage_code`] and
//! [`Upgradable::up_deploy_code`] in a [batch transaction] in case no staging duration is set.
//! Since [`Upgradable::up_deploy_code`] returns a promise that ultimately deploys the new contract
//! code, a theoretical risk remains. However, the [time between scheduling and execution] of a
//! promise hardly allows an attacker to exploit a vulnerability: they would have to retrieve the
//! bytes of the staged code, reverse engineer the new contract, build an exploit and finally
//! execute it. Therefore, we consider that risk of an exploit in case of a batched upgrade
//! negligible.
//!
//! Another defense mechanism is staging encrypted code, though this requires your own
//! implementation of the trait `Upgradable`. The default implementation provided by
//! `near-plugins-derive` does not support it.
//!
//! [example contract]: ../../near-plugins-derive/tests/contracts/upgradable/src/lib.rs
//! [state migration]: https://docs.near.org/develop/upgrade#migrating-the-state
//! [batch transaction]: https://docs.near.org/concepts/basics/transactions/overview
//! [time between scheduling and execution]: https://docs.near.org/sdk/rust/promises/intro
use crate::events::{AsEvent, EventMetadata};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{AccountId, CryptoHash, Gas, NearToken, Promise};

/// Trait describing the functionality of the _Upgradable_ plugin.
pub trait Upgradable {
    /// Returns the storage prefix for slots related to upgradable.
    ///
    /// Attribute `storage_prefix` can be used to set a different prefix:
    ///
    /// ```ignore
    /// #[derive(Upgradable)]
    /// #[upgradable(storage_prefix="CUSTOM_KEY")]
    /// struct Contract { /* ... */}
    /// ```
    fn up_storage_prefix(&self) -> &'static [u8];

    /// Returns all staging durations and timestamps.
    fn up_get_delay_status(&self) -> UpgradableDurationStatus;

    /// Allows an authorized account to stage code to be potentially deployed later. It sets the
    /// staging timestamp, which is the earliest time at which `code` may be deployed. The staging
    /// timestamp is calculated as the block timestamp plus the staging duration. Any code that was
    /// staged previously is discarded.
    ///
    /// If `code` is empty, previously staged code and the corresponding staging timestamp are
    /// removed.
    ///
    /// In the default implementation, this method is protected by access control provided by the
    /// `AccessControllable` plugin. The roles which may successfully call this method are
    /// specified via the `code_stagers` field of the `Upgradable` macro's `access_control_roles`
    /// attribute. The example contract (accessible via the `README`) shows how access control roles
    /// can be defined and passed on to the `Upgradable` macro.
    fn up_stage_code(&mut self, code: Vec<u8>);

    /// Returns the staged code.
    fn up_staged_code(&self) -> Option<Vec<u8>>;

    /// Returns the hash of the staged code
    fn up_staged_code_hash(&self) -> Option<CryptoHash>;

    /// Allows an authorized account to deploy the staged code. It panics if no code is staged.
    ///
    /// # Attaching a function call
    ///
    /// If `function_call_args` are provided, code is deployed in a batch promise that contains the
    /// `DeployContractAction` followed by `FunctionCallAction`. In case the function call fails,
    /// the deployment is rolled back and the initial code remains active. For this purpose,
    /// batching the actions mentioned above is required due to the [asynchronous design] of NEAR.
    ///
    /// Attaching a function call can be useful, for example, if deploying the staged code requires
    /// [state migration]. It can be achieved by calling a migration function defined in the new
    /// version of the contract. A failure during state migration can leave the contract in a broken
    /// state, which is avoided by the rollback mechanism described above.
    ///
    /// # Removal of staged code
    ///
    /// After deployment, staged code remains in storage. It is not removed automatically as this
    /// would cost extra gas and therefore increase the risk of the transaction hitting NEAR's gas
    /// limit. Moreover, in case the deployment is rollbacked due to a failure in the attached
    /// function call, the staged code might still be required.
    ///
    /// Once staged code is no longer needed, it can be removed by passing the appropriate arguments
    /// to [`Self::up_stage_code`]. Removing staged code allows to [unstake tokens] that are storage
    /// staked.
    ///
    /// It is recommended to remove staged code as soon as possible to avoid deploying code and
    /// executing an attached function call multiple times. Using batch transaction for this purpose
    /// can be dangerous. Since `up_deploy_code` returns a promise, there can be unexpected outcomes
    /// when it is combined in a batch transaction with another function call that removes code from
    /// storage. This is demonstrated in the `Upgradable` test
    /// `test_deploy_code_in_batch_transaction_pitfall`.
    ///
    /// # Permissions
    ///
    /// In the default implementation, this method is protected by access control provided by the
    /// `AccessControllable` plugin. The roles which may successfully call this method are
    /// specified via the `code_deployers` field of the `Upgradable` macro's `access_control_roles`
    /// attribute. The example contract (accessible via the `README`) shows how access control roles
    /// can be defined and passed on to the `Upgradable` macro.
    ///
    /// [asynchronous design]: https://docs.near.org/concepts/basics/transactions/overview
    /// [state migration]: https://docs.near.org/develop/upgrade#migrating-the-state
    /// [storage staked]: https://docs.near.org/concepts/storage/storage-staking#btw-you-can-remove-data-to-unstake-some-tokens
    fn up_deploy_code(&mut self, function_call_args: Option<FunctionCallArgs>) -> Promise;

    /// Initializes the duration of the delay for deploying the staged code. It defaults to zero if
    /// code is staged before the staging duration is initialized. Once the staging duration has
    /// been initialized, this method panics. For subsequent updates of the staging duration,
    /// [`Self::up_stage_update_staging_duration`] and [`Self::up_apply_update_staging_duration`]
    /// can be used.
    ///
    /// In the default implementation, this method is protected by access control provided by the
    /// `AccessControllable` plugin. The roles which may successfully call this method are
    /// specified via the `duration_initializers` field of the `Upgradable` macro's
    /// `access_control_roles` attribute. The example contract (accessible via the `README`) shows
    /// how access control roles can be defined and passed on to the `Upgradable` macro.
    fn up_init_staging_duration(&mut self, staging_duration: near_sdk::Duration);

    /// Allows an authorized account to stage an update of the staging duration. It panics if the
    /// staging duration was not previously initialized with [`Self::up_init_staging_duration`]. It
    /// sets the timestamp for the new staging duration, which is the earliest time at which the
    /// update may be applied. The new staging duration timestamp is calculated as the block
    /// timestamp plus the current staging duration.
    ///
    /// In the default implementation, this method is protected by access control provided by the
    /// `AccessControllable` plugin. The roles which may successfully call this method are specified
    /// via the `duration_update_stagers` field of the `Upgradable` macro's `access_control_roles`
    /// attribute. The example contract (accessible via the `README`) shows how access control roles
    /// can be defined and passed on to the `Upgradable` macro.
    fn up_stage_update_staging_duration(&mut self, staging_duration: near_sdk::Duration);

    /// Allows an authorized account to apply the staged update of the staging duration. It fails if
    /// no staging duration update is staged.
    ///
    /// In the default implementation, this method is protected by access control provided by the
    /// `AccessControllable` plugin. The roles which may successfully call this method are specified
    /// via the `duration_update_appliers` field of the `Upgradable` macro's `access_control_roles`
    /// attribute. The example contract (accessible via the `README`) shows how access control roles
    /// can be defined and passed on to the `Upgradable` macro.
    fn up_apply_update_staging_duration(&mut self);
}

#[derive(Deserialize, Serialize)]
#[allow(clippy::module_name_repetitions)]
pub struct UpgradableDurationStatus {
    pub staging_duration: Option<near_sdk::Duration>,
    pub staging_timestamp: Option<near_sdk::Timestamp>,
    pub new_staging_duration: Option<near_sdk::Duration>,
    pub new_staging_duration_timestamp: Option<near_sdk::Timestamp>,
}

/// Specifies a function call to be appended to the actions of a promise via
/// [`near_sdk::Promise::function_call`]).
#[derive(Deserialize, Serialize, Debug)]
pub struct FunctionCallArgs {
    /// The name of the function to call.
    pub function_name: String,
    /// The arguments to pass to the function.
    pub arguments: Vec<u8>,
    /// The amount of tokens to transfer to the receiver.
    pub amount: NearToken,
    /// The gas limit for the function call.
    pub gas: Gas,
}

/// Event emitted when the code is staged
#[derive(Serialize, Clone)]
struct StageCode {
    /// The account which staged the code.
    by: AccountId,
    /// The hash of the code that was staged.
    code_hash: CryptoHash,
}

impl AsEvent<Self> for StageCode {
    fn metadata(&self) -> EventMetadata<Self> {
        EventMetadata {
            standard: "Upgradable".to_string(),
            version: "1.0.0".to_string(),
            event: "stage_code".to_string(),
            data: Some(self.clone()),
        }
    }
}

/// Event emitted when the code is deployed
#[derive(Serialize, Clone)]
struct DeployCode {
    /// The account that deployed the code.
    by: AccountId,
    /// The hash of the code that was deployed.
    code_hash: CryptoHash,
}

impl AsEvent<Self> for DeployCode {
    fn metadata(&self) -> EventMetadata<Self> {
        EventMetadata {
            standard: "Upgradable".to_string(),
            version: "1.0.0".to_string(),
            event: "deploy_code".to_string(),
            data: Some(self.clone()),
        }
    }
}
