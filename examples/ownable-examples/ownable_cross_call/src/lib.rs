use near_plugins::Ownable;
use near_sdk::{env, near_bindgen};
use near_sdk::ext_contract;
use near_plugins_derive::only;
use borsh::{BorshSerialize, BorshDeserialize};

#[ext_contract(ext_counter)]
pub trait ExtCounter {
    fn protected_self(&mut self);
    fn protected_owner(&mut self);
}

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

  #[only(owner)]
  pub fn cross_call_owner_self(&mut self) {
      ext_counter::ext(env::current_account_id()).protected_self();
  }

  #[only(self)]
  pub fn cross_call_self_owner(&mut self) {
      ext_counter::ext(env::current_account_id()).protected_owner();
  }

  #[only(owner)]
  pub fn cross_call_owner_owner(&mut self) {
      ext_counter::ext(env::current_account_id()).protected_owner();
  }

  #[only(self)]
  pub fn cross_call_self_self(&mut self) {
      ext_counter::ext(env::current_account_id()).protected_self();
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

    const WASM_FILEPATH: &str = "../../target/wasm32-unknown-unknown/release/ownable_cross_call.wasm";

    #[tokio::test]
    async fn base_scenario() {
        let (contract_holder, contract) = get_contract(WASM_FILEPATH).await;

        assert!(call!(contract,"new").await);
        let next_owner = get_subaccount(&contract_holder, "next_owner").await;
        assert!(call!(contract, "owner_set", &json!({"owner": next_owner.id()})).await);
        let current_owner: Option::<AccountId> = view!(contract, "owner_get");
        assert_ne!(current_owner.clone().unwrap().as_str(), contract_holder.id().as_str());
        assert_eq!(current_owner.unwrap().as_str(), next_owner.id().as_str());

        assert!(call!(&next_owner, contract, "cross_call_owner_self").await);
        assert!(call!(&next_owner, contract, "cross_call_owner_owner").await);
        assert!(!call!(&next_owner, contract, "cross_call_self_self").await);
        assert!(!call!(&next_owner, contract, "cross_call_self_owner").await);

        check_counter(&contract, 1).await;

        assert!(!call!(contract, "cross_call_owner_self").await);
        assert!(!call!(contract, "cross_call_owner_owner").await);
        assert!(call!(contract, "cross_call_self_self").await);
        assert!(call!(contract, "cross_call_self_owner").await);

        check_counter(&contract, 2).await;
    }
}
