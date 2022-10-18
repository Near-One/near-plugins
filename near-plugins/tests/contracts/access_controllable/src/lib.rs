use near_plugins::{access_control, access_control_any, AccessControlRole, AccessControllable};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, log, near_bindgen, AccountId};
use std::collections::HashMap;

#[derive(AccessControlRole, Deserialize, Serialize, Copy, Clone)]
#[serde(crate = "near_sdk::serde")]
pub enum Role {
    LevelA,
    LevelB,
    LevelC,
}

#[access_control(role_type(Role))]
#[near_bindgen]
#[derive(Default, BorshDeserialize, BorshSerialize)]
pub struct StatusMessage {
    records: HashMap<AccountId, String>,
}

#[near_bindgen]
impl StatusMessage {
    // Adding an initial super-admin can be done via trait method
    // `AccessControllable::acl_init_super_admin`, which is automatically
    // implemented and exported for the contract by `#[access_controllable]`.
    //
    // Once an account is (super-)admin, it may add other admins and grant
    // roles.
    //
    // In addition, there are internal `*_unchecked` methods for example:
    //
    // ```
    // self.__acl.add_admin_unchecked(role, account_id);
    // self.__acl.grant_role_unchecked(role, account_id);
    // ```
    //
    // **Attention**: Contracts should call `__acl.*_unchecked` methods only
    // from within methods with attribute `#[init]` or `#[private]`.

    #[payable]
    pub fn set_status(&mut self, message: String) {
        let account_id = env::signer_account_id();
        log!("{} set_status with message {}", account_id, message);
        self.records.insert(account_id, message);
    }

    pub fn get_status(&self, account_id: AccountId) -> Option<String> {
        log!("get_status for account_id {}", account_id);
        self.records.get(&account_id).cloned()
    }

    #[access_control_any(roles(Role::LevelA, Role::LevelC))]
    pub fn restricted_greeting(&self) -> String {
        "hello world".to_string()
    }

    // In addition, `AccessControllable` trait methods can be called directly:
    //
    // ```
    // pub fn foo(&self) {
    //     let role = Role::LevelA;
    //     if self.acl_has_role(role.into(), &env::predecessor_account_id()) {
    //         // ..
    //     }
    // }
    // ```
}

/// Exposing internal methods to facilitate integration testing.
#[near_bindgen]
impl StatusMessage {
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
