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
    use serde_json::json;
    use near_sdk::AccountId;
    use near_plugins_test_utils::*;

    const WASM_FILEPATH: &str = "./target/wasm32-unknown-unknown/release/full_access_key_fallback_base.wasm";

    #[tokio::test]
    async fn base_scenario() {
        let (mut contract_holder, contract) = get_contract_testnet(WASM_FILEPATH).await;

        assert!(call!(contract,"new").await);

        check_counter(&contract, 0).await;

        let next_owner = get_subaccount(&contract_holder, "next_owner").await;
        assert!(call!(contract, "owner_set", &json!({"owner": next_owner.id()})).await);
        let current_owner: Option::<AccountId> = view!(contract, "owner_get");
        assert_eq!(current_owner.unwrap().as_str(), next_owner.id().as_str());

        assert!(call!(&contract_holder, contract, "protected_self").await);

        check_counter(&contract, 1).await;

        contract_holder.set_secret_key(next_owner.secret_key().clone());

        assert!(call!(&next_owner, contract, "attach_full_access_key",  &json!({"public_key": next_owner.secret_key().public_key()})).await);

        assert!(call!(contract, "protected_self").await);

        check_counter(&contract, 2).await;
    }

    #[tokio::test]
    #[should_panic(expected = "AccessKeyNotFound")]
    async fn base_panic_on_wrong_key() {
        let (mut contract_holder, contract) = get_contract_testnet(WASM_FILEPATH).await;

        assert!(call!(contract, "new").await);
        let next_owner = get_subaccount(&contract_holder, "next_owner").await;
        assert!(call!(contract, "owner_set", &json!({"owner": next_owner.id()})).await);

        assert!(call!(&contract_holder, contract, "protected_self").await);
        contract_holder.set_secret_key(next_owner.secret_key().clone());

        call!(&contract_holder, contract, "protected_self").await;
    }
}
