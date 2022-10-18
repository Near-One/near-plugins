use near_plugins::{Ownable, Upgradable};
use near_sdk::near_bindgen;
use borsh::{BorshSerialize, BorshDeserialize};

#[near_bindgen]
#[derive(Ownable, Upgradable, Default, BorshSerialize, BorshDeserialize)]
struct Counter1 {
  counter: u64,
}

#[near_bindgen]
impl Counter1 {
  /// Specify the owner of the contract in the constructor
  #[init]
  pub fn new() -> Self {
      let mut contract = Self { counter: 0 };
      contract.owner_set(Some(near_sdk::env::predecessor_account_id()));
      contract
  }

  pub fn inc1(&mut self) {
      self.counter += 1;
  }

  pub fn get_counter(&self) -> u64 {
      self.counter
  }
}


#[cfg(test)]
mod tests {
    use serde_json::json;
    use workspaces::{Account, Contract};
    use tokio::runtime::Runtime;
    use near_plugins_test_utils::*;

    const WASM_FILEPATH: &str = "./target/wasm32-unknown-unknown/release/upgradable_base.wasm";
    const WASM_FILEPATH_SECOND: &str = "../upgradable_base_second/target/wasm32-unknown-unknown/release/upgradable_base_second.wasm";

    fn get_contract() -> (Account, Contract) {
        let rt = Runtime::new().unwrap();
        let worker = rt.block_on(workspaces::testnet()).unwrap();

        let owner = rt.block_on(worker.dev_create_account()).unwrap();

        let wasm = std::fs::read(WASM_FILEPATH).unwrap();
        let contract = rt.block_on(owner.deploy(&wasm)).unwrap().unwrap();

        (owner, contract)
    }

    fn call_borsh_arg(contract: &Contract, method_name: &str, args: Vec<u8>) -> bool {
        let rt = Runtime::new().unwrap();

        rt.block_on(
            contract.call(method_name)
                .args_borsh(args)
                .max_gas()
                .transact()
        ).unwrap().is_success()
    }

    //https://docs.near.org/sdk/rust/promises/deploy-contract
    #[test]
    fn base_scenario() {
        let (_, contract) = get_contract();
        assert!(call!(contract,"new"));

        assert!(call!(contract, "inc1"));
        check_counter(&contract, 1);

        assert!(!call!(contract, "inc2"));
        check_counter(&contract, 1);

        let wasm = std::fs::read(WASM_FILEPATH_SECOND).unwrap();

        assert!(call_borsh_arg(&contract, "up_stage_code", wasm));
        assert!(call!(contract, "up_deploy_code"));
        check_counter(&contract, 1);

        assert!(!call!(contract, "inc1"));
        check_counter(&contract, 1);

        assert!(call!(contract, "inc2"));
        check_counter(&contract, 3);
    }
}
