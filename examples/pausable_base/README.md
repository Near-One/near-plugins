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
`pa_is_paused` is a _view_ method which returns if a feature is paused.

```shell
$ near view <CONTRACT_ACCOUNT> pa_is_paused '{"key": "increase_1"}'
View call: <CONTRACT_ACCOUNT>.pa_is_paused({"key": "increase_1"})
true
```

### pa_all_paused
`pa_all_paused` is a _view_ method which return the list of the all paused features.

```shell
$ near view <CONTRACT_ACCOUNT> pa_all_paused
View call: <CONTRACT_ACCOUNT>.pa_all_paused()
[ 'increase_1', 'increase_2' ]
```


### pa_pause_feature
`pa_pause_feature` is a method for pause specified feature. Can be run only by owner or self.

```shell
$ near call <CONTRACT_ACCOUNT> pa_pause_feature '{"key": "increase_1"}' --accountId <OWNER_ACCOUNT>
```

### pa_unpause_feature
`pa_unpause_feature` is a method for unpause specified feature. Can be run only by owner or self.

```shell
$ near call <CONTRACT_ACCOUNT> pa_unpause_feature '{"key": "increase_1"}' --accountId <OWNER_ACCOUNT>
```

## Preparation steps for demonstration
In that document we are providing some example of using contract with access control plugin. You also can explore the usage examples in the tests in `./src/lib.rs`. For running a tests please take a look to the **Test running instruction** section.

1. **Creating an account on testnet**
   For demonstration let's create 2 accounts: `<CONTRACT_ACCOUNT>`, `<ALICE_ACCOUNT>`
   ```shell
   $ near create-account <CONTRACT_ACCOUNT_NAME>.<MASTER_ACCOUNT_NAME>.testnet --masterAccount <MASTER_ACCOUNT_NAME>.testnet --initialBalance 10
   $ near create-account <ALICE_ACCOUNT_NAME>.<MASTER_ACCOUNT_NAME>.testnet --masterAccount <MASTER_ACCOUNT_NAME>.testnet --initialBalance 10
   ```

   In the next section we will refer to the `<CONTRACT_ACCOUNT_NAME>.<MASTER_ACCOUNT_NAME>.testnet` as `<CONTRACT_ACCOUNT>`,
   to the `<ALICE_ACCOUNT_NAME>.<MASTER_ACCOUNT_NAME>.testnet` as `<ALICE_ACCOUNT>`.

2. **Compile Contract to wasm file**
   For compiling the contract just run the `build.sh` script. The target file with compiled contract will be `./target/wasm32-unknown-unknown/release/pausable_base.wasm`

   ```shell
   $ ./build.sh
   ```

3. **Deploy and init a contract**
   ```shell
   $ near deploy --accountId <CONTRACT_ACCOUNT> --wasmFile ./target/wasm32-unknown-unknown/release/pausable_base.wasm --initFunction new --initArgs '{}'
   ```
   
## Example of using the contract with pausable plugin
### Simple pause and unpause function without name specification
At the beginning the `<CONTRACT_ACCOUNT>` both the self and the owner. 
`<ALICE_ACCOUNT>` doesn't have any specific rights. 

No features on pause:
```shell
$ near view <CONTRACT_ACCOUNT> pa_all_paused
View call: <CONTRACT_ACCOUNT>.pa_all_paused()
null
```

Alice can call `increase_1` function:
```shell
$ near call <CONTRACT_ACCOUNT> increase_1 --accountId <ALICE_ACCOUNT>
$ near view <CONTRACT_ACCOUNT> get_counter
1
```

#### Pause function
Self(or owner) pause function:
```shell
$ near call <CONTRACT_ACCOUNT> pa_pause_feature '{"key": "increase_1"}' --accountId <CONTRACT_ACCOUNT>
$ near view <CONTRACT_ACCOUNT> pa_all_paused
View call: <CONTRACT_ACCOUNT>.pa_all_paused()
[ 'increase_1' ]
$ near view <CONTRACT_ACCOUNT> pa_is_paused '{"key": "increase_1"}'
View call: <CONTRACT_ACCOUNT>.pa_is_paused({"key": "increase_1"})
true
```

Now Alice or even self cann't run `increase_1` function
```shell
$ near view <CONTRACT_ACCOUNT> get_counter
1
$ near call <CONTRACT_ACCOUNT> increase_1 --accountId <ALICE_ACCOUNT>
$ near call <CONTRACT_ACCOUNT> increase_1 --accountId <CONTRACT_ACCOUNT>
$ near view <CONTRACT_ACCOUNT> get_counter
1
```

#### Unpause function
Self(or owner) can unpause feature:
```shell
$ near call <CONTRACT_ACCOUNT> pa_unpause_feature '{"key": "increase_1"}' --accountId <CONTRACT_ACCOUNT>
$ near view <CONTRACT_ACCOUNT> pa_all_paused
View call: <CONTRACT_ACCOUNT>.pa_all_paused()
null
$ near view <CONTRACT_ACCOUNT> pa_is_paused '{"key": "increase_1"}'
View call: <CONTRACT_ACCOUNT>.pa_is_paused({"key": "increase_1"})
false
```

Now Alice can continue use the `increase_1` function
```shell
$ near view <CONTRACT_ACCOUNT> get_counter
1
$ near call <CONTRACT_ACCOUNT> increase_1 --accountId <ALICE_ACCOUNT>
$ near call <CONTRACT_ACCOUNT> increase_1 --accountId <CONTRACT_ACCOUNT>
$ near view <CONTRACT_ACCOUNT> get_counter
3
```

## Tests running instruction
Tests in `src/lib.rs` contain examples of interaction with a contract.

For running test:
1. Generate `wasm` file by running `build.sh` script. The target file will be `target/wasm32-unknown-unknown/release/pausable_base.wasm`
2. Run tests `cargo test`

```shell
$ ./build.sh
$ cargo test
```

For tests, we use `workspaces` library and `sandbox` environment for details you can explorer `../near-plugins-test-utils` crate
contract_account.