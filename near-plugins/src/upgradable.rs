//! # Upgradable
//!
//! Upgradable trait inspired by [NEP123](https://github.com/near/NEPs/pull/123).
//!
//! To upgrade the contract, first the code needs to be staged, and then it can be deployed.
//!
//! ## Default implementation
//!
//! Only owner or self can call [`Upgradable::up_stage_code`] and [`Upgradable::up_deploy_code`].
//!
//! There is no timer or staging duration implemented by default.
//!
//! ## Permissions
//!
//! Only an authorized account is allowed to call [`Upgradable::up_stage_code`] and
//! [`Upgradable::up_deploy_code`]. There may be several reasons to protect `deploy_code`. For
//! example if an upgrade requires migration or initialization. In that case, it is recommended to
//! run a batched transaction where [`Upgradable::up_deploy_code`] is called first, and then a
//! function that executes the migration or initialization.
//!
//! ## Stale staged code
//!
//! After the code is deployed, it should be removed from staging. This will prevent old code with a
//! security vulnerability to be deployed.
//!
//! ## Upgrading code that contains a security vulnerability
//!
//! Once code is staged for an upgrade, it is publicly visible via [`Upgradable::up_staged_code`].
//! Staged code that fixes a security vulnerability might be discovered by an attacker who then
//! exploits the vulnerability before its fix is deployed.
//!
//! To avoid that, the upgrade can be executed by calling [`Upgradable::up_stage_code`] and
//! [`Upgradable::up_deploy_code`] in a [batch transaction]. Since [`Upgradable::up_deploy_code`]
//! returns a promise that ultimately deploys the new contract code, a theoretical risk remains.
//! However, the [time between scheduling and execution] of a promise hardly allows an attacker to
//! exploit a vulnerability: they would have to retrieve the bytes of the staged code, reverse
//! engineer the new contract, build an exploit and finally execute it. Therefore, we consider that
//! risk of an exploit in case of a batched upgrade negligible.
//!
//! Another defense mechanism is staging encrypted code, though this requires your own
//! implementation of the trait `Upgradable`. The default implementation provided by
//! `near-plugins-derive` does not support it.
//!
//! [batch transaction]: https://docs.near.org/concepts/basics/transactions/overview
//! [time between scheduling and execution]: https://docs.near.org/sdk/rust/promises/intro
use crate::events::{AsEvent, EventMetadata};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{AccountId, CryptoHash, Promise};

/// Trait describing the functionality of the _Upgradable_ plugin.
pub trait Upgradable {
    /// Returns the storage prefix for slots related to upgradable.
    fn up_storage_prefix(&self) -> &'static [u8];

    /// Returns all staging durations and timestamps.
    fn up_get_delay_status(&self) -> UpgradableDurationStatus;

    /// Allows authorized account to stage some code to be potentially deployed later.
    /// If a previous code was staged but not deployed, it is discarded.
    fn up_stage_code(&mut self, code: Vec<u8>);

    /// Returns staged code.
    fn up_staged_code(&self) -> Option<Vec<u8>>;

    /// Returns hash of the staged code
    fn up_staged_code_hash(&self) -> Option<CryptoHash>;

    /// Allows authorized account to deploy staged code. If no code is staged the method fails.
    fn up_deploy_code(&mut self) -> Promise;

    /// Initialize the duration of the delay for deploying the staged code.
    fn up_init_staging_duration(&mut self, staging_duration: near_sdk::Duration);

    /// Allows authorized account to stage update of the staging duration.
    fn up_stage_update_staging_duration(&mut self, staging_duration: near_sdk::Duration);

    /// Allows authorized account to apply the staging duration update.
    fn up_apply_update_staging_duration(&mut self);
}

#[derive(Deserialize, Serialize)]
pub struct UpgradableDurationStatus {
    pub staging_duration: Option<near_sdk::Duration>,
    pub staging_timestamp: Option<near_sdk::Timestamp>,
    pub new_staging_duration: Option<near_sdk::Duration>,
    pub new_staging_duration_timestamp: Option<near_sdk::Timestamp>,
}

/// Event emitted when the code is staged
#[derive(Serialize, Clone)]
struct StageCode {
    /// The account which staged the code.
    by: AccountId,
    /// The hash of the code that was staged.
    code_hash: CryptoHash,
}

impl AsEvent<StageCode> for StageCode {
    fn metadata(&self) -> EventMetadata<StageCode> {
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

impl AsEvent<DeployCode> for DeployCode {
    fn metadata(&self) -> EventMetadata<DeployCode> {
        EventMetadata {
            standard: "Upgradable".to_string(),
            version: "1.0.0".to_string(),
            event: "deploy_code".to_string(),
            data: Some(self.clone()),
        }
    }
}
