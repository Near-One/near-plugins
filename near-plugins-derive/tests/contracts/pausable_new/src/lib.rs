use near_plugins::{
    access_control, if_paused, pause, AccessControlRole, AccessControllable, Pausable,
};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, near, AccountId, PanicOnDefault};

/// Define roles for access control of `Pausable` features.
/// IMPORTANT: Keep the same order of existing variants to preserve permission mappings.
#[derive(AccessControlRole, Deserialize, Serialize, Copy, Clone)]
#[serde(crate = "near_sdk::serde")]
pub enum Role {
    /// Now only used for pausing features
    PauseManager,
    /// Existing roles kept in the same order  
    Unrestricted4Increaser,
    /// Existing roles kept in the same order
    Unrestricted4Decreaser,
    /// Existing roles kept in the same order
    Unrestricted4Modifier,
    /// Add new roles at the end
    UnpauseManager,
}

#[access_control(role_type(Role))]
#[near(contract_state)]
#[derive(Pausable, PanicOnDefault)]
#[pausable(pause_roles(Role::PauseManager), unpause_roles(Role::UnpauseManager))]
pub struct Counter {
    counter: u64,
}

#[near]
impl Counter {
    /// Constructor initializes the counter to 0 and sets up ACL.
    #[init]
    pub fn new(pause_manager: AccountId, unpause_manager: AccountId) -> Self {
        let mut contract = Self { counter: 0 };

        // Make the contract itself super admin
        near_sdk::require!(
            contract.acl_init_super_admin(env::current_account_id()),
            "Failed to initialize super admin",
        );

        // Grant roles to the provided accounts
        let result = contract.acl_grant_role(Role::PauseManager.into(), pause_manager);
        near_sdk::require!(Some(true) == result, "Failed to grant pause role");

        let result = contract.acl_grant_role(Role::UnpauseManager.into(), unpause_manager);
        near_sdk::require!(Some(true) == result, "Failed to grant unpause role");

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

    /// Migration function to maintain backward compatibility
    /// Grants UnpauseManager role to existing PauseManager accounts
    #[private]
    pub fn migrate_pause_unpause_roles(&mut self) {
        // Get all accounts with PauseManager role
        let pause_managers = self.acl_get_grantees("PauseManager".to_string(), 0, 100);

        // Grant UnpauseManager role to all existing PauseManager accounts
        for account_id in pause_managers {
            let result = self.acl_grant_role(Role::UnpauseManager.into(), account_id);
            near_sdk::require!(result.is_some(), "Failed to grant UnpauseManager role");
        }
    }
}
