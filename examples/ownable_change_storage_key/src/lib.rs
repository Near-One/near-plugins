use near_plugins::Ownable;
use near_sdk::near_bindgen;
use near_plugins_derive::only;
use borsh::{BorshSerialize, BorshDeserialize};

#[near_bindgen]
#[derive(Ownable, Default, BorshSerialize, BorshDeserialize)]
#[ownable(owner_storage_key="new_storage_key")]
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
    use serde_json::json;
    use near_plugins_test_utils::*;

    const WASM_FILEPATH: &str = "./target/wasm32-unknown-unknown/release/ownable_change_storage_key.wasm";

    #[test]
    fn check_owner_storage_key() {
        let (_, contract) = get_contract(WASM_FILEPATH);
        assert!(call!(contract,"new"));

        let owner_storage_key: Vec<u8> = view!(contract, "owner_storage_key");
        assert_eq!(owner_storage_key, "new_storage_key".as_bytes().to_vec());
    }
}
