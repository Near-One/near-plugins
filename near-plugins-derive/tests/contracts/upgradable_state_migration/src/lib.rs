//! A simple contract to be deployed via `Upgradable`. It requires [state migration].
//!
//! [state migration]: https://docs.near.org/develop/upgrade#migrating-the-state

use near_plugins::{access_control, AccessControlRole, AccessControllable, Upgradable};
use near_sdk::borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, near, PanicOnDefault};

/// Roles correspond to those defined in the initial contract `../upgradable`, to make permissions
/// granted before the upgrade remain valid.
#[derive(AccessControlRole, Deserialize, Serialize, Copy, Clone)]
#[serde(crate = "near_sdk::serde")]
pub enum Role {
    DAO,
    CodeStager,
    CodeDeployer,
    DurationManager,
}

/// The struct differs from the one defined in the initial contract `../upgradable`, hence [state
/// migration] is required.
///
/// [state migration]: https://docs.near.org/develop/upgrade#migrating-the-state
#[access_control(role_type(Role))]
#[near(contract_state)]
#[derive(Upgradable, PanicOnDefault)]
#[upgradable(access_control_roles(
    code_stagers(Role::CodeStager, Role::DAO),
    code_deployers(Role::CodeDeployer, Role::DAO),
    duration_initializers(Role::DurationManager, Role::DAO),
    duration_update_stagers(Role::DurationManager, Role::DAO),
    duration_update_appliers(Role::DurationManager, Role::DAO),
))]
pub struct Contract {
    is_migrated: bool,
}

#[near]
impl Contract {
    /// Migrates state from [`OldContract`] to [`Contract`].
    ///
    /// It follows the state migration pattern described [here].
    ///
    /// [here]: https://docs.near.org/develop/upgrade#migrating-the-state
    #[private]
    #[init(ignore_state)]
    pub fn migrate() -> Self {
        // Ensure old state can be read and deserialized.
        let _: OldContract = env::state_read().expect("Should be able to load old state");

        Self { is_migrated: true }
    }

    /// A migration method that fails on purpose to test the rollback mechanism of
    /// `Upgradable::up_deploy_code`.
    #[private]
    #[init(ignore_state)]
    pub fn migrate_with_failure() -> Self {
        env::panic_str("Failing migration on purpose");
    }

    /// This method is _not_ defined in the initial contract, so calling it successfully proves the
    /// contract defined in this file was deployed and the old state was migrated.
    pub fn is_migrated(&self) -> bool {
        self.is_migrated
    }
}

/// Corresponds to the state defined in the initial `../upgradable` contract.
#[derive(BorshDeserialize)]
#[borsh(crate = "near_sdk::borsh")]
pub struct OldContract;
