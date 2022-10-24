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

## The contract methods description
### up_storage_key
`up_storage_key` is a _view_ method which return a key of storage slot for stage code.
By default `b"__CODE__"` is used. For changing the attribute `upgradable` can be used.

```shell
$ near view <CONTRACT_ACCOUNT> up_storage_key
View call: <CONTRACT_ACCOUNT>.up_storage_key()
[
  95, 95, 80, 65, 85,
  83, 69, 68, 95, 95
]
$ python3
>>> print(' '.join(str(b) for b in bytes("__CODE__", 'utf8')))
95 95 67 79 68 69 95 95
```

Example of changing paused storage key:
```rust
#[near_bindgen]
#[derive(Ownable, Upgradable, Default, BorshSerialize, BorshDeserialize)]
#[upgradable(code_storage_key="OTHER_CODE_STORAGE_KEY")]
struct Counter {
  counter: u64,
}
```

### up_stage_code
`up_stage_code` method to stage some code to be potentially deployed later. If a previous code was staged but not deployed, it is discarded. 
Method can be called only by owner

```shell
$ export CODE=$(cat ../upgradable_base_second/target/wasm32-unknown-unknown/release/upgradable_base_second.wasm | xxd -ps | sed -z 's/\n//g')
$ near call <CONTRACT_ACCOUNT> up_stage_code --base64 $CODE  --accountId <CONTRACT_ACCOUNT>
```

But it doesn't work in that way because we cann't provide in Bash so long args... So, probable here we can't use just NEAR CLI for interaction with contract :(

For running `up_satge_code` take a look on `src/up_stage_code.rs` script.
```shell
cargo run -- "<PATH_TO_KEY_FOR_CONTRACT_ACCOUNT>"
```

### up_staged_code
`up_staged_code` a _view_ method which returns a staged code.

```shell
$ near call <CONTRACT_ACCOUNT> up_staged_code --accountId <CONTRACT_ACCOUNT>
```

### up_staged_code_hash
`up_staged_code_hash` a _view_ method which returns the hash of the staged code.

```shell
$ near view <CONTRACT_ACCOUNT> up_staged_code_hash
View call: <CONTRACT_ACCOUNT>.up_staged_code_hash()
[
   63,  26, 245, 200, 217,  12, 109,  77,
   40, 222,  40, 173, 192, 197,  28, 236,
  231, 239,  19, 223, 212,  99,  98, 228,
  162, 119,  89,  37, 250, 173,  87,   5
]
```

### up_deploy_code
`up_deploy_code` method deploy a staged code. If no code is staged the method fails.
Method can be called only by owner

```shell
$ near call <CONTRACT_ACCOUNT> up_deploy_code --accountId <CONTRACT_ACCOUNT>
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

2. **Compile 2 Contract to wasm file**
   We should compile 2 contracts: `upgradable_base`, `upgradable_base_second`.

   ```shell
   $ ./build.sh
   $ cd ../upgradable_base_second
   $ ./build.sh
   $ cd ../upgradable_base
   ```

   The contracts will be compiled into `./target/wasm32-unknown-unknown/release/upgradable_base.wasm`, `../upgradable_base_second/target/wasm32-unknown-unknown/release/upgradable_base_second.wasm`

3. **Deploy and init a contract**
   ```shell
   $ near deploy --accountId <CONTRACT_ACCOUNT> --wasmFile ./target/wasm32-unknown-unknown/release/upgradable_base.wasm --initFunction new --initArgs '{}'
   ```

## Example of using the contract with upgradable plugin
### Upgrade contract
Currently on `<CONTRACT_ACCOUNT>` contract `Counter1` is deployed, and we would like to upgrade it to `Counter2`. 

#### Increment counter on first contract
```shell
$ near view <CONTRACT_ACCOUNT> get_counter
0
$ near call <CONTRACT_ACCOUNT> inc1 --accountId <ALICE_ACCOUNT>
$ near view <CONTRACT_ACCOUNT> get_counter
1
$ near call <CONTRACT_ACCOUNT> inc2 --accountId <ALICE_ACCOUNT>
$ near view <CONTRACT_ACCOUNT> get_counter
1
```

#### Stage new contract
```shell
cargo run -- "<PATH_TO_KEY_FOR_CONTRACT_ACCOUNT>"
```

#### Deploy new contract
```shell
$ near call <CONTRACT_ACCOUNT> up_deploy_code --accountId <CONTRACT_ACCOUNT>
```

#### Increment counter on second contract
```shell
$ near view <CONTRACT_ACCOUNT> get_counter
1
$ near call <CONTRACT_ACCOUNT> inc1 --accountId <ALICE_ACCOUNT>
$ near view <CONTRACT_ACCOUNT> get_counter
1
$ near call <CONTRACT_ACCOUNT> inc2 --accountId <ALICE_ACCOUNT>
$ near view <CONTRACT_ACCOUNT> get_counter
2
```

## Tests running instruction
Tests in `src/lib.rs` contain examples of interaction with a contract.

For running test:
1. Generate `wasm` file by running `build.sh` script. The target file will be `target/wasm32-unknown-unknown/release/upgradable_base.wasm`
2. Run tests `cargo test`

```shell
$ ./build.sh
$ cargo test
```

For tests, we use `workspaces` library and `sandbox` environment for details you can explorer `../near-plugins-test-utils` crate
contract_account.