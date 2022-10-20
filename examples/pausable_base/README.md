# Example of using Pausable plugin
Allow contracts to implement an emergency stop mechanism that can be triggered by an authorized account. Pauses can be used granularly to only limit certain features.

Contract example using Pausable plugin. Note that it requires the contract to be Ownable.

```rust
use near_plugins::Ownable;
use near_plugins::Pausable;
use near_sdk::near_bindgen;
use near_plugins_derive::{pause, if_paused};
use borsh::{BorshSerialize, BorshDeserialize};

#[near_bindgen]
#[derive(Ownable, Pausable, Default, BorshSerialize, BorshDeserialize)]
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

    /// Function can be paused using feature name "increase_1" or "ALL" like:
    /// `contract.pa_pause_feature("increase_1")` or `contract.pa_pause_feature("ALL")`
    ///
    /// If the function is paused, all calls to it will fail. Even calls started from owner or self.
    #[pause]
    pub fn increase_1(&mut self) {
        self.counter += 1;
    }

    /// Similar to `#[pause]` but use an explicit name for the feature. In this case the feature to be paused
    /// is named "Increase by two". Note that trying to pause it using "increase_2" will not have any effect.
    ///
    /// This can be used to pause a subset of the methods at once without requiring to use "ALL".
    #[pause(name = "Increase by two")]
    pub fn increase_2(&mut self) {
        self.counter += 2;
    }

    /// Similar to `#[pause]` but owner or self can still call this method. Any subset of {self, owner} can be specified.
    #[pause(except(owner, self))]
    pub fn increase_4(&mut self) {
        self.counter += 4;
    }

    /// This method can only be called when "increase_1" is paused. Use this macro to create escape hatches when some
    /// features are paused. Note that if "ALL" is specified the "increase_1" is considered to be paused.
    #[if_paused(name = "increase_1")]
    pub fn decrease_1(&mut self) {
        self.counter -= 1;
    }

    /// Custom use of pause features. Only allow increasing the counter using `careful_increase` if it is below 10.
    pub fn careful_increase(&mut self) {
        if self.counter >= 10 {
            assert!(
                !self.pa_is_paused("INCREASE_BIG".to_string()),
                "Method paused for large values of counter"
            );
        }

        self.counter += 1;
    }

    pub fn get_counter(&self) -> u64 {
        self.counter
    }
}
```

## The contract methods description
### pa_storage_key
`pa_storage_key` is a _view_ method which returns the key of storage slot with list of paused features.
By default `b"__PAUSED__"` is used. For changing the attribute `pausable` can be used.

```shell
$ near view <CONTRACT_ACCOUNT> pa_storage_key
View call: <CONTRACT_ACCOUNT>.pa_storage_key()
[
  95, 95, 80, 65, 85,
  83, 69, 68, 95, 95
]
$ python3
>>> print(' '.join(str(b) for b in bytes("__PAUSED__", 'utf8')))
95 95 80 65 85 83 69 68 95 95
```

Example of changing paused storage key:
```rust
#[near_bindgen]
#[derive(Ownable, Pausable, Default, BorshSerialize, BorshDeserialize)]
#[pausable(paused_storage_key="OTHER_PAUSED_STORAGE_KEY")]
struct Counter {
  counter: u64,
}
```

### pa_is_paused
`pa_is_paused` is a _view_ which returns if a feature is paused.

```shell
$ near view <CONTRACT_ACCOUNT> pa_is_paused '{"key": "increase_1"}'
View call: <CONTRACT_ACCOUNT>.pa_is_paused({"key": "increase_1"})
false
```

### pa_all_paused

### pa_pause_feature

### pa_unpause_feature