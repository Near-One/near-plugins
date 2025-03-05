use near_plugins::{
    access_control, if_paused, pause, AccessControlRole, AccessControllable, Pausable,
};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, near, AccountId, PanicOnDefault};

/// Define roles for access control of `Pausable` features.
#[derive(AccessControlRole, Deserialize, Serialize, Copy, Clone)]
#[serde(crate = "near_sdk::serde")]
pub enum Role {
    /// The pause manager in the old style can both pause and unpause
    PauseManager,
    /// For testing except functionality
    Unrestricted4Increaser,
    /// For testing except functionality
    Unrestricted4Decreaser,
    /// For testing except functionality
    Unrestricted4Modifier,
}

#[access_control(role_type(Role))]
#[near(contract_state)]
#[derive(Pausable, PanicOnDefault)]
#[pausable(pause_roles(Role::PauseManager), unpause_roles(Role::PauseManager))]
pub struct Counter {
    counter: u64,
}

#[near]
impl Counter {
    /// Constructor initializes the counter to 0 and sets up ACL.
    #[init]
    pub fn new(pause_manager: AccountId) -> Self {
        let mut contract = Self { counter: 0 };

        // Make the contract itself super admin
        near_sdk::require!(
            contract.acl_init_super_admin(env::current_account_id()),
            "Failed to initialize super admin",
        );

        // Grant role to the provided account
        let result = contract.acl_grant_role(Role::PauseManager.into(), pause_manager);
        near_sdk::require!(Some(true) == result, "Failed to grant pause role");

        contract
    }

    /// Returns the current counter value
    #[pause]
    pub fn get_counter(&self) -> u64 {
        self.counter
    }

    /// Increments the counter - can be paused
    #[pause]
    pub fn increment(&mut self) {
        self.counter += 1;
    }

    /// Similar to `#[pause]` but use an explicit name for the feature.
    #[pause(name = "Increase by two")]
    pub fn increase_2(&mut self) {
        self.counter += 2;
    }

    /// Similar to `#[pause]` but roles passed as argument may still successfully call this method
    /// even when the corresponding feature is paused.
    #[pause(except(roles(Role::Unrestricted4Increaser, Role::Unrestricted4Modifier)))]
    pub fn increase_4(&mut self) {
        self.counter += 4;
    }

    /// This method can only be called when "increment" is paused.
    #[if_paused(name = "increment")]
    pub fn decrease_1(&mut self) {
        self.counter -= 1;
    }

    /// For verifying that an account has a specific role
    pub fn has_role(&self, role: String, account_id: AccountId) -> bool {
        self.acl_has_role(role, account_id)
    }
}
