# Example of using Upgradable plugin

Allows a contract to be upgraded by owner without having a Full Access Key.

Contract example using Upgradable plugin. Note that it requires the contract to be Ownable.

```rust
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
```

The second example contract for upgrading you can find in the `../upgradable_base_second/` folder.

The second contract: 
```rust
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
```

To upgrade the contract first call up_stage_code passing the binary as first argument serialized as borsh. Then call up_deploy_code. This functions must be called from the owner.