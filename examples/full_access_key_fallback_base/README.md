# Example of using Full Access Key Fallback plugin

Allows an authorized account to attach a Full Access Key to the contract.

Contract example using Full Access Key Fallback plugin. Note that it requires the contract to be Ownable.

```rust
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
```

## The contract methods description
### attach_full_access_key
