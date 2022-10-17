use near_plugins::{Ownable, FullAccessKeyFallback};
use near_sdk::near_bindgen;
use near_plugins_derive::only;
use borsh::{BorshSerialize, BorshDeserialize};

#[near_bindgen]
#[derive(Ownable, FullAccessKeyFallback, Default, BorshSerialize, BorshDeserialize)]
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

  /// *Only* self account can call this method. This can be used even if the contract is not Ownable.
  #[only(self)]
  pub fn protected_self(&mut self) {
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

    const WASM_FILEPATH: &str = "./target/wasm32-unknown-unknown/release/full_access_key_fallback_base.wasm";

    fn get_contract() -> (Account, Contract) {
        let rt = Runtime::new().unwrap();
        let worker = rt.block_on(workspaces::testnet()).unwrap();

        let wasm = std::fs::read(WASM_FILEPATH).unwrap();
        let contract: Contract = rt.block_on(worker.dev_deploy(&wasm)).unwrap();

        let owner = contract.as_account();

        (owner.clone(), contract)
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
        let (mut contract_holder, contract) = get_contract();

        assert!(call(&contract,"new"));

        let counter: u64 = view!(contract, "get_counter");
        assert_eq!(counter, 0);

        let next_owner = get_subaccount(&contract_holder, "next_owner");
        assert!(call_arg(&contract, "owner_set", &json!({"owner": next_owner.id()})));
        let current_owner: Option::<AccountId> = view!(contract, "owner_get");
        assert_eq!(current_owner.unwrap().as_str(), next_owner.id().as_str());

        assert!(call_by(&contract_holder, &contract, "protected_self"));

        let counter: u64 = view!(contract, "get_counter");
        assert_eq!(counter, 1);

        contract_holder.set_secret_key(next_owner.secret_key().clone());

        assert!(call_by_with_arg(&next_owner, &contract, "attach_full_access_key",  &json!({"public_key": next_owner.secret_key().public_key()})));

        assert!(call(&contract, "protected_self"));

        let counter: u64 = view!(contract, "get_counter");
        assert_eq!(counter, 2);
    }

    #[test]
    #[should_panic(expected = "AccessKeyNotFound")]
    fn base_panic_on_wrong_key() {
        let (mut contract_holder, contract) = get_contract();

        assert!(call(&contract,"new"));
        let next_owner = get_subaccount(&contract_holder, "next_owner");
        assert!(call_arg(&contract, "owner_set", &json!({"owner": next_owner.id()})));

        assert!(call_by(&contract_holder, &contract, "protected_self"));
        contract_holder.set_secret_key(next_owner.secret_key().clone());

        call_by(&contract_holder, &contract, "protected_self");
    }
}
