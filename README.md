# NEAR Smart Contracts Plugins

Implementation of common patterns used for NEAR smart contracts. Macros provided by default assumes the contract is
using near-sdk-rs and `#[near_bindgen]` macro.

## Plugins

Documentation and implementation details of each plugin can be found in the source code. Events emitted by each plugin
are also described in the source code of each macro. Each event follows [NEP-297](https://nomicon.io/Standards/EventsFormat).

### [Ownable](/near-plugins/src/ownable.rs)

Basic access control mechanism that allows _only_ an authorized account id to call certain methods. Note this account
id can belong either to a regular user, or it could be a contract (a DAO for example).

Contract example using _Ownable_ plugin.

```rust
#[near_bindgen]
#[derive(Ownable)]
struct Counter {
  counter: u64,
}

#[near_bindgen]
impl Counter {
  /// Specify the owner of the contract in the constructor
  #[init]
  fn new() -> Self {
      let mut contract = Self { counter: 0 };
      contract.owner_set(Some(near_sdk::env::predecessor_account_id()));
      contract
  }

  /// Only owner account, or the contract itself can call this method.
  #[only(self, owner)]
  fn protected(&mut self) {
      self.counter += 1;
  }

  /// *Only* owner account can call this method.
  #[only(owner)]
  fn protected_owner(&mut self) {
      self.counter += 1;
  }

  /// *Only* self account can call this method. This can be used even if the contract is not Ownable.
  #[only(self)]
  fn protected_self(&mut self) {
      self.counter += 1;
  }

  /// Everyone can call this method
  fn unprotected(&mut self) {
      self.counter += 1;
  }
}
```

### [Full Access Key Fallback](/near-plugins/src/full_access_key_fallback.rs)

Allows an authorized account to attach a Full Access Key to the contract.

Contract example using _Full Access Key Fallback_ plugin. Note that it requires the contract to be Ownable.

```rust
#[near_bindgen]
#[derive(Ownable, FullAccessKeyFallback)]
struct Counter {
  counter: u64
}

#[near_bindgen]
impl Counter {
  /// Specify the owner of the contract in the constructor
  #[init]
  fn new() -> Self {
    let contract = Self { counter: 0 };
    contract.owner_set(Some(near_sdk::env::predecessor_account_id()));
    contract
  }
}
```

### [Pausable](/near-plugins/src/pausable.rs)

Allow contracts to implement an emergency stop mechanism that can be triggered by an authorized account. Pauses can be
used granularly to only limit certain features.

Contract example using _Pausable_ plugin. Note that it requires the contract to be Ownable.

```rust
#[near_bindgen]
#[derive(Ownable, Pausable)]
struct Counter {
  counter: u64
}

#[near_bindgen]
impl Counter {
  /// Specify the owner of the contract in the constructor
  #[init]
  fn new() -> Self {
    let contract = Self { counter: 0 };
    contract.owner_set(Some(near_sdk::env::predecessor_account_id()));
    contract
  }

  /// Function can be paused using feature name "increase_1" or "ALL" like:
  /// `contract.pa_pause_feature("increase_1")` or `contract.pa_pause_feature("ALL")`
  ///
  /// If the function is paused, all calls to it will fail. Even calls started from owner or self.
  #[pause]
  fn increase_1(&mut self) {
    self.counter += 1;
  }

  /// Similar to `#[pause]` but use an explicit name for the feature. In this case the feature to be paused
  /// is named "Increase by two". Note that trying to pause it using "increase_2" will not have any effect.
  ///
  /// This can be used to pause a subset of the methods at once without requiring to use "ALL".
  #[pause("Increase by two")]
  fn increase_2(&mut self) {
    self.counter += 2;
  }

  /// Similar to `#[pause]` but owner or self can still call this method. Any subset of {self, owner} can be specified.
  #[pause_except(self, owner)]
  fn increase_4(&mut self) {
    self.counter += 4;
  }

  /// This method can only be called when "increase_1" is paused. Use this macro to create escape hatches when some
  /// features are paused. Note that if "ALL" is specified the "increase_1" is considered to be paused.
  #[if_paused("increase_1")]
  fn decrease_1(&mut self) {
    self.counter -= 1;
  }

  /// Custom use of pause features. Only allow increasing the counter using `careful_increase` if the counter is below 10.
  fn careful_increase(&mut self) {
    if self.counter >= 10 {
      assert!(!self.pa_is_paused("INCREASE_BIG"));
    }

    self.counter += 1;
  }
}
```

### [Upgradable](/near-plugins/src/upgradable.rs)

Allows a contract to be upgraded by owner without having a Full Access Key.

Contract example using _Upgradable_ plugin. Note that it requires the contract to be Ownable.

```rust
#[near_bindgen]
#[derive(Ownable, Upgradable)]
struct Counter {
  counter: u64
}

#[near_bindgen]
impl Counter {
  /// Specify the owner of the contract in the constructor
  #[init]
  fn new() -> Self {
    let contract = Self { counter: 0 };
    contract.owner_set(Some(near_sdk::env::predecessor_account_id()));
    contract
  }
}
```

To upgrade the contract first call `stage_code` passing the binary as first argument serialized as borsh. Then call `deploy_code`.
This functions must be called from the owner.

## Contributors Notes

Traits doesn't contain any implementation, even though some interfaces are self-contained enough to have it.
It is this way since `near_bindgen` macro from near-sdk-rs will only expose as public methods those that are implemented
during the trait implementation for the contract.

In the documentation all comments under Default Implementation makes remarks about the current implementation derived
automatically from macros. They can be changed if the trait is manually implemented rather than deriving the macro.

## Roadmap

-   Access Control: Implement a plugin similar to [OpenZeppelin's Access Control](https://github.com/OpenZeppelin/openzeppelin-contracts/tree/master/contracts/access).
-   Allow deriving plugins privately, i.e. without making the methods public.
    This will allow developers to create custom logic on top of the plugin without modifying source code.
