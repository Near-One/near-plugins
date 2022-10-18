use near_plugins::AccessControllable;
use near_plugins::AccessControlRole;
use near_plugins_derive::access_control;
use near_plugins_derive::access_control_any;
use near_sdk::near_bindgen;
use borsh::{BorshSerialize, BorshDeserialize};
use near_plugins::events::AsEvent;
use near_sdk::env;

#[derive(AccessControlRole, Clone, Copy)]
pub enum Positions {
    LevelA,
    LevelB,
    LevelC
}

#[near_bindgen]
#[access_control(role_type="Positions")]
#[derive(Default, BorshSerialize, BorshDeserialize)]
struct Counter {
  counter: u64,
}

#[near_bindgen]
impl Counter {
    #[init]
    pub fn new() -> Self {
        let mut contract: Counter = Self{
            counter: 0,
            __acl: __Acl::default(),
        };

        contract.__acl.init_super_admin(&near_sdk::env::predecessor_account_id());

        contract
    }

    pub fn unprotected(&mut self) {
        self.counter += 1;
    }

    #[access_control_any(roles(Positions::LevelA))]
    pub fn level_a_incr(&mut self) {
        self.counter += 1;
    }

    #[access_control_any(roles(Positions::LevelA, Positions::LevelB))]
    pub fn level_ab_incr(&mut self) {
        self.counter += 1;
    }

    pub fn get_counter(&self) -> u64 {
        self.counter
    }
}


#[cfg(test)]
mod tests {
    use serde_json::json;
    use crate::Positions;
    use near_plugins_test_utils::*;

    const WASM_FILEPATH: &str = "./target/wasm32-unknown-unknown/release/access_controllable_base.wasm";

    #[test]
    fn base_scenario() {
        let (contract_holder, contract) = get_contract(WASM_FILEPATH);
        assert!(call!(contract,"new"));

        let counter: u64 = view!(contract, "get_counter");
        assert_eq!(counter, 0);

        assert!(call!(contract, "unprotected"));

        let counter: u64 = view!(contract, "get_counter");
        assert_eq!(counter, 1);

        let alice = get_subaccount(&contract_holder, "alice");

        let is_super_admin: bool = view!(contract, "acl_is_super_admin", &json!({"account_id": alice.id()}));
        assert!(!is_super_admin);

        assert!(!call!(&alice, contract, "level_a_incr"));
        let counter: u64 = view!(contract, "get_counter");
        assert_eq!(counter, 1);

        assert!(call!(contract, "acl_grant_role", &json!({"role": String::from(Positions::LevelA), "account_id": alice.id()})));

        let alice_has_role: bool = view!(contract, "acl_has_role", &json!({"role": String::from(Positions::LevelA), "account_id": alice.id()}));
        assert!(alice_has_role);

        assert!(call!(&alice, contract, "level_a_incr"));

        let counter: u64 = view!(contract, "get_counter");
        assert_eq!(counter, 2);


        let bob = get_subaccount(&contract_holder, "bob");
        assert!(call!(contract, "acl_add_admin", &json!({"role": String::from(Positions::LevelA), "account_id": bob.id()})));

        let bob_is_admin: bool = view!(contract, "acl_is_admin", &json!({"role": String::from(Positions::LevelA), "account_id": bob.id()}));
        assert!(bob_is_admin);

        assert!(!call!(&bob, contract, "level_a_incr"));

        let counter: u64 = view!(contract, "get_counter");
        assert_eq!(counter, 2);

        assert!(call!(&alice, contract, "level_ab_incr"));
        let counter: u64 = view!(contract, "get_counter");
        assert_eq!(counter, 3);

        assert!(!call!(&bob, contract, "level_ab_incr"));
        let counter: u64 = view!(contract, "get_counter");
        assert_eq!(counter, 3);

        assert!(call!(contract, "acl_grant_role", &json!({"role": String::from(Positions::LevelB), "account_id": bob.id()})));
        assert!(call!(&bob, contract, "level_ab_incr"));
        let counter: u64 = view!(contract, "get_counter");
        assert_eq!(counter, 4);

        assert!(!call!(&bob, contract, "level_a_incr"));
        let counter: u64 = view!(contract, "get_counter");
        assert_eq!(counter, 4);
    }
}
