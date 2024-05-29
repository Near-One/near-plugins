use near_plugins::{
    access_control, if_paused, pause, AccessControlRole, AccessControllable, Pausable,
};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, near, AccountId, PanicOnDefault};

/// Define roles for access control of `Pausable` features. Accounts which are
/// granted a role are authorized to execute the corresponding action.
#[derive(AccessControlRole, Deserialize, Serialize, Copy, Clone)]
#[serde(crate = "near_sdk::serde")]
pub enum Role {
    /// May pause and unpause features.
    PauseManager,
    /// May call `increase_4` even when it is paused.
    Unrestricted4Increaser,
    /// May call `decrease_4` even when `increase_4` is not paused.
    Unrestricted4Decreaser,
    /// May always call both `increase_4` and `decrease_4`.
    Unrestricted4Modifier,
}

#[access_control(role_type(Role))]
#[near(contract_state)]
#[derive(Pausable, PanicOnDefault)]
#[pausable(manager_roles(Role::PauseManager))]
pub struct Counter {
    counter: u64,
}

#[near]
impl Counter {
    /// Permissons for `AccessControllable` can be initialized in the constructor. Here we are:
    ///
    /// * Making the contract itself super admin.
    /// * Granting `Role::PauseManager` to the account id `pause_manager`.
    ///
    /// For a general overview of access control, please refer to the `AccessControllable` plugin.
    #[init]
    pub fn new(pause_manager: AccountId) -> Self {
        let mut contract = Self {
            counter: 0,
        };

        // Make the contract itself super admin. This allows us to grant any role in the
        // constructor.
        near_sdk::require!(
            contract.acl_init_super_admin(env::current_account_id()),
            "Failed to initialize super admin",
        );

        // Grant `Role::PauseManager` to the provided account.
        let result = contract.acl_grant_role(Role::PauseManager.into(), pause_manager);
        near_sdk::require!(Some(true) == result, "Failed to grant role");

        contract
    }

    /// Returns the value of the counter.
    pub fn get_counter(&self) -> u64 {
        self.counter
    }

    /// Function can be paused using feature name "increase_1" or "ALL" like:
    /// `contract.pa_pause_feature("increase_1")` or `contract.pa_pause_feature("ALL")`
    ///
    /// If the function is paused, all calls to it will fail. Even calls initiated by accounts which
    /// are access control super admin or role grantee.
    #[pause]
    pub fn increase_1(&mut self) {
        self.counter += 1;
    }

    /// Similar to `#[pause]` but use an explicit name for the feature. In this case the feature to
    /// be paused is named "Increase by two". Note that trying to pause it using "increase_2" will
    /// not have any effect.
    ///
    /// This can be used to pause a subset of the methods at once without requiring to use "ALL".
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

    /// This method can only be called when "increase_1" is paused. Use this macro to create escape
    /// hatches when some features are paused. Note that if "ALL" is specified the "increase_1" is
    /// considered to be paused.
    #[if_paused(name = "increase_1")]
    pub fn decrease_1(&mut self) {
        self.counter -= 1;
    }

    /// Similar to `#[if_paused]` but roles passed as argument may successfully call the method even
    /// when the feature is _not_ paused.
    #[if_paused(name = "increase_4", except(roles(Role::Unrestricted4Decreaser)))]
    pub fn decrease_4(&mut self) {
        self.counter -= 4;
    }

    /// Custom use of pause features. Only allow increasing the counter using `careful_increase` if
    /// it is below 3.
    pub fn careful_increase(&mut self) {
        if self.counter >= 3 {
            assert!(
                !self.pa_is_paused("increase_big".to_string()),
                "Method paused for large values of counter"
            );
        }

        self.counter += 1;
    }
}
