use near_plugins::AccessControllable;
use near_plugins::AccessControlRole;
use near_plugins_derive::access_control;
use near_plugins_derive::access_control_any;
use near_sdk::near_bindgen;
use borsh::{BorshSerialize, BorshDeserialize};
use near_sdk::env;

/// All types of access groups
#[derive(AccessControlRole, Clone, Copy)]
pub enum UsersGroups {
    GroupA,
    GroupB,
}

#[near_bindgen]
#[access_control(role_type(UsersGroups))]
#[derive(Default, BorshSerialize, BorshDeserialize)]
struct Counter {
  counter: u64,
}

#[near_bindgen]
impl Counter {
    /// In the constructor we set up a super admin,
    /// which can control the member lists of all user groups
    #[init]
    pub fn new() -> Self {
        let mut contract: Counter = Self{
            counter: 0,
            __acl: __Acl::default(),
        };

        contract.__acl.init_super_admin(&near_sdk::env::predecessor_account_id());

        contract
    }

    /// unprotected function, every one can call this function
    pub fn unprotected(&mut self) {
        self.counter += 1;
    }

    /// only the users from GroupA can call this method
    #[access_control_any(roles(UsersGroups::GroupA))]
    pub fn level_a_incr(&mut self) {
        self.counter += 1;
    }

    /// only the users from GroupA or GroupB can call this method
    #[access_control_any(roles(UsersGroups::GroupA, UsersGroups::GroupB))]
    pub fn level_ab_incr(&mut self) {
        self.counter += 1;
    }


    /// view method for get current counter value, every one can use it
    pub fn get_counter(&self) -> u64 {
        self.counter
    }
}


#[cfg(test)]
mod tests {
    use serde_json::json;
    use crate::UsersGroups;
    use near_plugins_test_utils::*;

    const WASM_FILEPATH: &str = "../../target/wasm32-unknown-unknown/release/access_controllable_base.wasm";

    #[tokio::test]
    async fn base_scenario() {
        let (contract_holder, contract) = get_contract(WASM_FILEPATH).await;
        assert!(call!(contract,"new").await);

        check_counter(&contract, 0).await;

        assert!(call!(contract, "unprotected").await);

        check_counter(&contract, 1).await;

        let alice = get_subaccount(&contract_holder, "alice").await;

        let is_super_admin: bool = view!(contract, "acl_is_super_admin", &json!({"account_id": alice.id()}));
        assert!(!is_super_admin);

        assert!(!call!(&alice, contract, "level_a_incr").await);
        check_counter(&contract, 1).await;

        assert!(call!(contract, "acl_grant_role", &json!({"role": String::from(UsersGroups::GroupA), "account_id": alice.id()})).await);

        let alice_has_role: bool = view!(contract, "acl_has_role", &json!({"role": String::from(UsersGroups::GroupA), "account_id": alice.id()}));
        assert!(alice_has_role);

        assert!(call!(&alice, contract, "level_a_incr").await);

        check_counter(&contract, 2).await;

        let bob = get_subaccount(&contract_holder, "bob").await;
        assert!(call!(contract, "acl_add_admin", &json!({"role": String::from(UsersGroups::GroupA), "account_id": bob.id()})).await);

        let bob_is_admin: bool = view!(contract, "acl_is_admin", &json!({"role": String::from(UsersGroups::GroupA), "account_id": bob.id()}));
        assert!(bob_is_admin);

        assert!(!call!(&bob, contract, "level_a_incr").await);

        check_counter(&contract, 2).await;

        assert!(call!(&alice, contract, "level_ab_incr").await);
        check_counter(&contract, 3).await;

        assert!(!call!(&bob, contract, "level_ab_incr").await);
        check_counter(&contract, 3).await;

        assert!(call!(contract, "acl_grant_role", &json!({"role": String::from(UsersGroups::GroupB), "account_id": bob.id()})).await);
        assert!(call!(&bob, contract, "level_ab_incr").await);
        check_counter(&contract, 4).await;

        assert!(!call!(&bob, contract, "level_a_incr").await);
        check_counter(&contract, 4).await;
    }
}
