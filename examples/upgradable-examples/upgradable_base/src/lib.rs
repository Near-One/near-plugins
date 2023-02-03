use near_plugins::{Ownable, Upgradable};
use near_sdk::near_bindgen;
use borsh::{BorshSerialize, BorshDeserialize};

#[near_bindgen]
#[derive(Ownable, Upgradable, Default, BorshSerialize, BorshDeserialize)]
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
        contract.up_init_staging_duration(std::time::Duration::from_secs(60).as_nanos().try_into().unwrap()); // 1 minute
        contract
    }

    #[cfg(feature = "counter1")]
    pub fn inc1(&mut self) {
        self.counter += 1;
    }

    #[cfg(feature = "counter2")]
    pub fn inc2(&mut self) {
        self.counter += 2;
    }

    pub fn get_counter(&self) -> u64 {
        self.counter
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use workspaces::{Account, Contract};
    use near_plugins_test_utils::*;

    const WASM_FILEPATH: &str = "../../target/wasm32-unknown-unknown/release/upgradable_base.wasm";
    const WASM_FILEPATH_SECOND: &str = "../../target/wasm32-unknown-unknown/release/upgradable_base_second.wasm";

    async fn get_contract() -> (Account, Contract) {
        let worker = workspaces::testnet().await.unwrap();

        let owner = worker.dev_create_account().await.unwrap();

        let wasm = std::fs::read(WASM_FILEPATH).unwrap();
        let contract = owner.deploy(&wasm).await.unwrap().unwrap();

        (owner, contract)
    }

    async fn call_method_with_borsh_args(contract: &Contract, method_name: &str, args: Vec<u8>) -> bool {
        contract.call(method_name)
                .args_borsh(args)
                .max_gas()
                .transact()
        .await.unwrap().is_success()
    }

    //https://docs.near.org/sdk/rust/promises/deploy-contract
    #[tokio::test]
    async fn base_scenario() {
        let (_, contract) = get_contract().await;
        assert!(call!(contract,"new").await);

        assert!(call!(contract, "inc1").await);
        check_counter(&contract, 1).await;

        assert!(!call!(contract, "inc2").await);
        check_counter(&contract, 1).await;

        let wasm = std::fs::read(WASM_FILEPATH_SECOND).unwrap();

        assert!(call_method_with_borsh_args(&contract, "up_stage_code", wasm).await);
        assert!(call!(contract, "up_deploy_code").await);
        check_counter(&contract, 1).await;

        assert!(!call!(contract, "inc1").await);
        check_counter(&contract, 1).await;

        assert!(call!(contract, "inc2").await);
        check_counter(&contract, 3).await;
    }
}
