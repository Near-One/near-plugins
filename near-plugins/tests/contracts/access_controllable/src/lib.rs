use near_plugins::events::AsEvent;
use near_plugins::{access_control, access_control_any, AccessControlRole, AccessControllable};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{env, log, near_bindgen, AccountId};
use std::collections::HashMap;
use std::convert::TryFrom;

#[derive(AccessControlRole, Copy, Clone)]
pub enum Role {
    LevelA,
    LevelB,
    LevelC,
}

#[access_control(role_type = "Role")]
#[near_bindgen]
#[derive(Default, BorshDeserialize, BorshSerialize)]
pub struct StatusMessage {
    records: HashMap<AccountId, String>,
}

#[near_bindgen]
impl StatusMessage {
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
