use near_plugins::Ownable;
use near_plugins::Pausable;
use near_sdk::near_bindgen;
use near_plugins_derive::only;
use near_plugins_derive::{pause, if_paused};
use borsh::{BorshSerialize, BorshDeserialize};

#[near_bindgen]
#[derive(Ownable, Pausable, Default, BorshSerialize, BorshDeserialize)]
struct Counter {
  counter: u64,
}

#[near_bindgen]
impl Counter {
    /// Specify the owner of the contract in the constructor
    #[init]
    pub fn new() -> Self {
        let mut contract = Self { counter: 0 };
        contract.owner_set(Some(near_sdk::env::predecessor_account_id()));
        contract
    }

    /// Function can be paused using feature name "increase_1" or "ALL" like:
    /// `contract.pa_pause_feature("increase_1")` or `contract.pa_pause_feature("ALL")`
    ///
    /// If the function is paused, all calls to it will fail. Even calls started from owner or self.
    #[pause]
    pub fn increase_1(&mut self) {
        self.counter += 1;
    }

    /// Similar to `#[pause]` but use an explicit name for the feature. In this case the feature to be paused
    /// is named "Increase by two". Note that trying to pause it using "increase_2" will not have any effect.
    ///
    /// This can be used to pause a subset of the methods at once without requiring to use "ALL".
    #[pause(name = "Increase by two")]
    pub fn increase_2(&mut self) {
        self.counter += 2;
    }

    /// Similar to `#[pause]` but owner or self can still call this method. Any subset of {self, owner} can be specified.
    #[pause(except(owner, self))]
    pub fn increase_4(&mut self) {
        self.counter += 4;
    }

    /// This method can only be called when "increase_1" is paused. Use this macro to create escape hatches when some
    /// features are paused. Note that if "ALL" is specified the "increase_1" is considered to be paused.
    #[if_paused(name = "increase_1")]
    pub fn decrease_1(&mut self) {
        self.counter -= 1;
    }

    /// Custom use of pause features. Only allow increasing the counter using `careful_increase` if it is below 10.
    pub fn careful_increase(&mut self) {
        if self.counter >= 10 {
            assert!(
                !self.pa_is_paused("INCREASE_BIG".to_string()),
                "Method paused for large values of counter"
            );
        }

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
    use workspaces::result::{ExecutionResult, ExecutionSuccess, ValueOrReceiptId};

    const WASM_FILEPATH: &str = "./target/wasm32-unknown-unknown/release/pausable_base.wasm";

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

    fn call_by_with_arg(account: &Account, contract: &Contract, method_name: &str, args: &serde_json::Value) -> bool {
        let rt = Runtime::new().unwrap();

        rt.block_on(
            account.call(contract.id(), method_name)
                .args_json(args)
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
        }
    }

    #[test]
    fn base_scenario() {
        let (contract_holder, contract) = get_contract();

        assert!(call(&contract,"new"));

        let next_owner = get_subaccount(&contract_holder, "next_owner");
        assert!(call_arg(&contract, "owner_set", &json!({"owner": next_owner.id()})));
        let current_owner: Option::<AccountId> = view!(contract, "owner_get");
        assert_eq!(current_owner.unwrap().as_str(), next_owner.id().as_str());

        let alice = get_subaccount(&contract_holder, "alice");

        assert!(call_by(&alice, &contract, "increase_1"));

        let counter: u64 = view!(contract, "get_counter");
        assert_eq!(counter, 1);

        assert!(!call_by_with_arg(&alice, &contract, "pa_pause_feature", &json!({"key": "increase_1"})));
        assert!(call_by_with_arg(&next_owner, &contract, "pa_pause_feature", &json!({"key": "increase_1"})));

        assert!(!call_by(&alice, &contract, "increase_1"));

        let counter: u64 = view!(contract, "get_counter");
        assert_eq!(counter, 1);

        assert!(!call_by(&next_owner, &contract, "increase_1"));
        let counter: u64 = view!(contract, "get_counter");
        assert_eq!(counter, 1);

        assert!(!call_by_with_arg(&contract_holder, &contract, "pa_unpause_feature", &json!({"key": "increase_1"})));
        assert!(call_by_with_arg(&next_owner, &contract, "pa_unpause_feature", &json!({"key": "increase_1"})));

        assert!(call_by(&alice, &contract, "increase_1"));

        let counter: u64 = view!(contract, "get_counter");
        assert_eq!(counter, 2);
    }
}