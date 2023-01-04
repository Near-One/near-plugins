use near_plugins::{access_control, access_control_any, AccessControlRole, AccessControllable};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, near_bindgen, AccountId, PanicOnDefault};
use std::collections::HashMap;

/// Roles are represented by enum variants.
///
/// Deriving `AccessControlRole` ensures `Role` can be used in `AccessControllable`.
#[derive(AccessControlRole, Deserialize, Serialize, Copy, Clone)]
#[serde(crate = "near_sdk::serde")]
pub enum Role {
    /// Grantees of this role may call the contract method `increase`.
    Increaser,
    /// Grantees of this role may call the contract method `skip_one`.
    Skipper,
    /// Grantees of this role may call the contract method `reset`.
    Resetter,
}

/// Pass `Role` to the `access_controllable` macro.
#[access_control(role_type(Role))]
#[near_bindgen]
#[derive(PanicOnDefault, BorshDeserialize, BorshSerialize)]
pub struct Counter {
    counter: u64,
}

#[near_bindgen]
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
        let mut contract = Self {
            counter: 0,
            // Initialize `AccessControllable` plugin state.
            __acl: Default::default(),
        };

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
            // contract.__acl.add_admin_unchecked(role, account_id);
            // contract.__acl.grant_role_unchecked(role, account_id);
            // ```
            //
            // **Attention**: for security reasons, `__acl.*_unchecked` methods should only be called
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
    /// Only an account that was granted either `Role::Increaser` or `Role::Skipper` may
    /// successfully call this method.
    #[access_control_any(roles(Role::Increaser, Role::Skipper))]
    pub fn skip_one(&mut self) -> u64 {
        self.counter += 2;
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
    /// here `__acl`.
    ///
    /// This function shows how these methods can be exposed on the contract.
    /// Usually this should involve security checks, for example requiring the
    /// caller to be a super admin.
    pub fn add_super_admin(&mut self, account_id: AccountId) -> bool {
        near_sdk::require!(
            self.acl_is_super_admin(env::predecessor_account_id()),
            "Only super admins are allowed to add other super admins."
        );
        self.__acl.add_super_admin_unchecked(&account_id)
    }
}

/// Exposing internal methods to facilitate integration testing.
#[near_bindgen]
impl Counter {
    #[private]
    pub fn acl_add_super_admin_unchecked(&mut self, account_id: AccountId) -> bool {
        self.__acl.add_super_admin_unchecked(&account_id)
    }

    #[private]
    pub fn acl_revoke_super_admin_unchecked(&mut self, account_id: AccountId) -> bool {
        self.__acl.revoke_super_admin_unchecked(&account_id)
    }

    #[private]
    pub fn acl_revoke_role_unchecked(&mut self, role: Role, account_id: AccountId) -> bool {
        self.__acl.revoke_role_unchecked(role.into(), &account_id)
    }

    #[private]
    pub fn acl_add_admin_unchecked(&mut self, role: Role, account_id: AccountId) -> bool {
        self.__acl.add_admin_unchecked(role, &account_id)
    }

    #[private]
    pub fn acl_revoke_admin_unchecked(&mut self, role: Role, account_id: AccountId) -> bool {
        self.__acl.revoke_admin_unchecked(role, &account_id)
    }

    #[private]
    pub fn acl_grant_role_unchecked(&mut self, role: Role, account_id: AccountId) -> bool {
        self.__acl.grant_role_unchecked(role, &account_id)
    }
}
