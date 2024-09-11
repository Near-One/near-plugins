use near_plugins::{access_control, access_control_any, AccessControlRole, AccessControllable};
use near_sdk::borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, near, AccountId, PanicOnDefault};
use std::collections::HashMap;

/// Roles are represented by enum variants.
///
/// Deriving `AccessControlRole` ensures `Role` can be used in `AccessControllable`.
#[derive(AccessControlRole, Deserialize, Serialize, Copy, Clone)]
#[serde(crate = "near_sdk::serde")]
pub enum Role {
    /// Grantees may call contract methods increasing the counter by up to _two_ at once.
    ByMax2Increaser,
    /// Grantees may call contract methods increasing the counter by up to _three_ at once.
    ByMax3Increaser,
    /// Grantees of this role may call the contract method `reset`.
    Resetter,
}

/// Pass `Role` to the `access_controllable` macro.
#[access_control(role_type(Role))]
#[near(contract_state)]
#[derive(PanicOnDefault)]
pub struct Counter {
    counter: u64,
}

#[near]
impl Counter {
    /// Constructor of the contract which optionally adds access control admins and grants roles if
    /// either of the maps passed as parameters contains account ids. In that case, the contract
    /// itself is made super admin, which permits it to add admins and grantees for every role.
    ///
    /// Both `admins` and `grantees` map the string representation of a role to an account id. With
    /// standard `serde` deserialization, the string representation of a role corresponds to the
    /// identifier of the enum variant, i.e. `"Updater"` for `Role::Updater`.
    #[init]
    pub fn new(admins: HashMap<String, AccountId>, grantees: HashMap<String, AccountId>) -> Self {
        let mut contract = Self { counter: 0 };

        if admins.len() > 0 || grantees.len() > 0 {
            // First we make the contract itself super admin to allow it adding admin and grantees.
            // That can be done via trait method `AccessControllable::acl_init_super_admin`, which is
            // automatically implemented and exported for the contract by `#[access_controllable]`.
            near_sdk::require!(
                contract.acl_init_super_admin(env::current_account_id()),
                "Failed to initialize super admin",
            );

            // Add admins.
            for (role, account_id) in admins.into_iter() {
                let result = contract.acl_add_admin(role, account_id);
                near_sdk::require!(Some(true) == result, "Failed to add admin");
            }

            // Grant roles.
            for (role, account_id) in grantees.into_iter() {
                let result = contract.acl_grant_role(role, account_id);
                near_sdk::require!(Some(true) == result, "Failed to grant role");
            }

            // Using internal `*_unchecked` methods is another option for adding (super) admins and
            // granting roles, for example:
            //
            // ```
            // contract.acl_get_or_init().add_admin_unchecked(role, account_id);
            // contract.acl_get_or_init().grant_role_unchecked(role, account_id);
            // ```
            //
            // **Attention**: for security reasons, `acl_get_or_init().*_unchecked` methods should only be called
            // from within methods with attribute `#[init]` or `#[private]`.
        }

        contract
    }

    /// Returns the current value of the counter.
    ///
    /// This method has no access control. Anyone can call it successfully.
    pub fn get_counter(&self) -> u64 {
        self.counter
    }

    /// Increases the counter by one and returns its new value.
    ///
    /// This method has no access control. Anyone can call it successfully.
    pub fn increase(&mut self) -> u64 {
        self.counter += 1;
        self.counter
    }

    /// Increases the counter by two and returns its new value.
    ///
    /// This method shows how to pass multiple `Role` variants to the `roles` attribute of
    /// `access_control_any`. It lets any account which was granted at least one of the specified
    /// roles call the method successfully. If the caller was not granted any of these roles, the
    /// method panics.
    #[access_control_any(roles(Role::ByMax2Increaser, Role::ByMax3Increaser))]
    pub fn increase_2(&mut self) -> u64 {
        self.counter += 2;
        self.counter
    }

    /// Increases the counter by three and returns its new value.
    ///
    /// Only an account that was granted `Role::ByMax3Increaser` may successfully call this method.
    #[access_control_any(roles(Role::ByMax3Increaser))]
    pub fn increase_3(&mut self) -> u64 {
        self.counter += 3;
        self.counter
    }

    /// Resets the counters value to zero.
    ///
    /// Only an account that was granted `Role:Resetter` may successfully call this method.
    #[access_control_any(roles(Role::Resetter))]
    pub fn reset(&mut self) {
        self.counter = 0;
    }

    /// The implementation of `AccessControllable` provided by `near-plugins`
    /// adds further methods to the contract which are not part of the trait.
    /// Most of them are implemented for the type that holds the plugin's state,
    /// which can be accessed with `self.acl_get_or_init()`.
    ///
    /// This function shows how these methods can be exposed on the contract.
    /// Usually this should involve security checks, for example requiring the
    /// caller to be a super admin.
    pub fn add_super_admin(&mut self, account_id: AccountId) -> bool {
        near_sdk::require!(
            self.acl_is_super_admin(env::predecessor_account_id()),
            "Only super admins are allowed to add other super admins."
        );
        self.acl_get_or_init()
            .add_super_admin_unchecked(&account_id)
    }
}

/// Exposing internal methods to facilitate integration testing.
#[near]
impl Counter {
    #[private]
    pub fn acl_add_super_admin_unchecked(&mut self, account_id: AccountId) -> bool {
        self.acl_get_or_init()
            .add_super_admin_unchecked(&account_id)
    }

    #[private]
    pub fn acl_revoke_super_admin_unchecked(&mut self, account_id: AccountId) -> bool {
        self.acl_get_or_init()
            .revoke_super_admin_unchecked(&account_id)
    }

    #[private]
    pub fn acl_revoke_role_unchecked(&mut self, role: Role, account_id: AccountId) -> bool {
        self.acl_get_or_init()
            .revoke_role_unchecked(role.into(), &account_id)
    }

    #[private]
    pub fn acl_add_admin_unchecked(&mut self, role: Role, account_id: AccountId) -> bool {
        self.acl_get_or_init()
            .add_admin_unchecked(role, &account_id)
    }

    #[private]
    pub fn acl_revoke_admin_unchecked(&mut self, role: Role, account_id: AccountId) -> bool {
        self.acl_get_or_init()
            .revoke_admin_unchecked(role, &account_id)
    }

    #[private]
    pub fn acl_grant_role_unchecked(&mut self, role: Role, account_id: AccountId) -> bool {
        self.acl_get_or_init()
            .grant_role_unchecked(role, &account_id)
    }
}
