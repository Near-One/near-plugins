use near_plugins::Ownable;
use near_sdk::near_bindgen;
use near_plugins_derive::only;
use borsh::{BorshSerialize, BorshDeserialize};

#[near_bindgen]
#[derive(Ownable, Default, BorshSerialize, BorshDeserialize)]
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

  /// Only owner account, or the contract itself can call this method.
  #[only(self, owner)]
  pub fn protected(&mut self) {
      self.counter += 1;
  }

  /// *Only* owner account can call this method.
  #[only(owner)]
  pub fn protected_owner(&mut self) {
      self.counter += 1;
  }

  /// *Only* self account can call this method. This can be used even if the contract is not Ownable.
  #[only(self)]
  pub fn protected_self(&mut self) {
      self.counter += 1;
  }

  /// Everyone can call this method
  pub fn unprotected(&mut self) {
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

    const WASM_FILEPATH: &str = "./target/wasm32-unknown-unknown/release/ownable_base.wasm";

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

        let current_owner: Option::<AccountId> = view!(contract, "owner_get");
        assert_eq!(current_owner.unwrap().as_str(), contract_holder.id().as_str());

        let counter: u64 = view!(contract, "get_counter");
        assert_eq!(counter, 0);

        assert!(call(&contract, "protected"));
        assert!(call(&contract, "protected_owner"));
        assert!(call(&contract, "protected_self"));
        assert!(call(&contract, "unprotected"));

        let counter: u64 = view!(contract, "get_counter");
        assert_eq!(counter, 4);

        let next_owner = get_subaccount(&contract_holder, "next_owner");
        assert!(!call_by(&next_owner, &contract, "protected"));
        assert!(!call_by(&next_owner, &contract, "protected_owner"));
        assert!(!call_by(&next_owner, &contract, "protected_self"));
        assert!(call_by(&next_owner, &contract, "unprotected"));

        let counter: u64 = view!(contract, "get_counter");
        assert_eq!(counter, 5);

        assert!(call_arg(&contract, "owner_set", &json!({"owner": next_owner.id()})));

        let current_owner: Option::<AccountId> = view!(contract, "owner_get");
        assert_ne!(current_owner.clone().unwrap().as_str(), contract_holder.id().as_str());
        assert_eq!(current_owner.unwrap().as_str(), next_owner.id().as_str());

        assert!(call_by(&next_owner, &contract, "protected"));
        assert!(call_by(&next_owner, &contract, "protected_owner"));
        assert!(!call_by(&next_owner, &contract, "protected_self"));
        assert!(call_by(&next_owner, &contract, "unprotected"));

        let counter: u64 = view!(contract, "get_counter");
        assert_eq!(counter, 8);

        assert!(call(&contract, "protected"));
        assert!(!call(&contract, "protected_owner"));
        assert!(call(&contract, "protected_self"));
        assert!(call(&contract, "unprotected"));

        let counter: u64 = view!(contract, "get_counter");
        assert_eq!(counter, 11);
    }

    #[test]
    fn null_owner() {
        let (contract_holder, contract) = get_contract();
        let rt = Runtime::new().unwrap();

        assert!(call(&contract,"new"));

        assert!(call_arg(&contract, "owner_set", &json!({"owner": Option::<AccountId>::None})));

        let current_owner: Option::<AccountId> = view!(contract, "owner_get");
        assert_eq!(current_owner, None);

        assert!(call(&contract, "protected"));
        assert!(!call(&contract, "protected_owner"));
        assert!(call(&contract, "protected_self"));
        assert!(call(&contract, "unprotected"));

        let counter: u64 = view!(contract, "get_counter");
        assert_eq!(counter, 3);

        assert!(call_arg(&contract, "owner_set", &json!({"owner": contract.id().as_str()})));
        assert!(call(&contract, "protected"));
        assert!(call(&contract, "protected_owner"));
        assert!(call(&contract, "protected_self"));
        assert!(call(&contract, "unprotected"));

        let counter: u64 = view!(contract, "get_counter");
        assert_eq!(counter, 7);
    }

    #[test]
    fn check_owner_storage_key() {
        let (contract_holder, contract) = get_contract();
        let rt = Runtime::new().unwrap();

        assert!(call(&contract,"new"));

        let owner_storage_key: Vec<u8> = view!(contract, "owner_storage_key");
        assert_eq!(owner_storage_key, "__OWNER__".as_bytes().to_vec());
    }
}
