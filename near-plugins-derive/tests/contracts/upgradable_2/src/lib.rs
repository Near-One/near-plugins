//! A simple contract to be deployed via `Upgradable`.

use near_plugins::{access_control, AccessControlRole, AccessControllable, Upgradable};
use near_sdk::borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{near, PanicOnDefault};

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

/// The struct is the same as in the initial contract `../upgradable`, so no [state migration] is
/// required.
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
pub struct Contract;

#[near]
impl Contract {
    /// A method that is _not_ defined in the initial contract, so its existence proves the
    /// contract defined in this file was deployed.
    pub fn is_upgraded() -> bool {
        true
    }
}
