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

    pub fn get_counter(&self) -> u64 {
        self.counter
    }
}


#[cfg(test)]
mod tests {
    use workspaces::{Account, Contract};
    use tokio::runtime::Runtime;
    use serde_json::{json, Value};
    use near_sdk::{AccountId, ONE_NEAR};
    use crate::Positions;
    use workspaces::result::{ExecutionResult, ExecutionSuccess, ValueOrReceiptId};

    const WASM_FILEPATH: &str = "./target/wasm32-unknown-unknown/release/access_controllable_base.wasm";

    fn get_contract() -> (Account, Contract) {
        let rt = Runtime::new().unwrap();
        let worker = rt.block_on(workspaces::sandbox()).unwrap();

        let owner = worker.root_account().unwrap();

        let wasm = std::fs::read(WASM_FILEPATH).unwrap();
        let contract = rt.block_on(owner.deploy(&wasm)).unwrap().unwrap();

        (owner, contract)
    }

    fn view(contract: &Contract, method_name: &str) -> Vec<u8> {
        let rt = Runtime::new().unwrap();

        rt.block_on(
            contract.view(method_name,
                          json!({}).to_string().into_bytes())
        ).unwrap().result
    }

    fn view_args(contract: &Contract, method_name: &str, args: &serde_json::Value) -> Vec<u8> {
        let rt = Runtime::new().unwrap();

        rt.block_on(
            contract.view(method_name,
                          args.to_string().into_bytes())
        ).unwrap().result
    }

    fn call(contract: &Contract, method_name: &str) -> bool {
        let rt = Runtime::new().unwrap();

        rt.block_on(
            contract.call(method_name)
                .max_gas()
                .transact()
        ).unwrap().is_success()
    }

    fn call_arg(contract: &Contract, method_name: &str, args: &serde_json::Value) -> bool {
        let rt = Runtime::new().unwrap();

        rt.block_on(
            contract.call(method_name)
                .args_json(args)
                .max_gas()
                .transact()
        ).unwrap().is_success()
    }

    fn call_by(account: &Account, contract: &Contract, method_name: &str) -> bool {
        let rt = Runtime::new().unwrap();

        rt.block_on(
            account.call(contract.id(),method_name)
                .max_gas()
                .transact()
        ).unwrap().is_success()
    }

    fn get_subaccount(account: &Account, new_account_name: &str) -> Account {
        let rt = Runtime::new().unwrap();

        rt.block_on(account.create_subaccount(new_account_name)
            .initial_balance(ONE_NEAR)
            .transact()).unwrap().unwrap()
    }

    macro_rules! view {
        ($contract:ident, $method_name:literal) => {
            serde_json::from_slice(&view(&$contract, $method_name)).unwrap()
        };
        ($contract:ident, $method_name:literal, $args:expr) => {
            serde_json::from_slice(&view_args(&$contract, $method_name, $args)).unwrap()
        };
    }

    #[test]
    fn base_scenario() {
        let (contract_holder, contract) = get_contract();
        assert!(call(&contract,"new"));

        let counter: u64 = view!(contract, "get_counter");
        assert_eq!(counter, 0);

        assert!(call(&contract, "unprotected"));

        let counter: u64 = view!(contract, "get_counter");
        assert_eq!(counter, 1);

        let alice = get_subaccount(&contract_holder, "alice");

        let is_super_admin: bool = view!(contract, "acl_is_super_admin", &json!({"account_id": alice.id()}));
        assert!(!is_super_admin);

        assert!(!call_by(&alice, &contract, "level_a_incr"));
        let counter: u64 = view!(contract, "get_counter");
        assert_eq!(counter, 1);

        assert!(call_arg(&contract, "acl_grant_role", &json!({"role": String::from(Positions::LevelA), "account_id": alice.id()})));

        let alice_has_role: bool = view!(contract, "acl_has_role", &json!({"role": String::from(Positions::LevelA), "account_id": alice.id()}));
        assert!(alice_has_role);

        assert!(call_by(&alice, &contract, "level_a_incr"));

        let counter: u64 = view!(contract, "get_counter");
        assert_eq!(counter, 2);


        let bob = get_subaccount(&contract_holder, "bob");
        assert!(call_arg(&contract, "acl_add_admin", &json!({"role": String::from(Positions::LevelA), "account_id": bob.id()})));

        let bob_is_admin: bool = view!(contract, "acl_is_admin", &json!({"role": String::from(Positions::LevelA), "account_id": bob.id()}));
        assert!(bob_is_admin);

        assert!(!call_by(&bob, &contract, "level_a_incr"));

        let counter: u64 = view!(contract, "get_counter");
        assert_eq!(counter, 2);
    }
}
