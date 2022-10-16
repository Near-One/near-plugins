use near_plugins::{Ownable, Upgradable};
use near_sdk::near_bindgen;
use borsh::{BorshSerialize, BorshDeserialize};

#[near_bindgen]
#[derive(Ownable, Upgradable, Default, BorshSerialize, BorshDeserialize)]
struct Counter2 {
  counter: u64,
}

#[near_bindgen]
impl Counter2 {
  #[init]
  pub fn new() -> Self {
      let mut contract = Self { counter: 0 };
      contract.owner_set(Some(near_sdk::env::predecessor_account_id()));
      contract
  }

  pub fn inc2(&mut self) {
      self.counter += 2;
  }

  pub fn get_counter(&self) -> u64 {
      self.counter
  }
}