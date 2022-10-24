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
`attach_full_access_key` - attach a new full access for the current account. 
Only owner of the contract can use this function. 

```shell
near call <CONTRACT_ACCOUNT> attach_full_access_key '{"public_key": "ed25519:ErVTCTvmepb4NDhQ7infTomkLVsd1iTWwLR84FBhV7UC"}' --accountId <OWNER_ACCOUNT>
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
   For compiling the contract just run the `build.sh` script. The target file with compiled contract will be `./target/wasm32-unknown-unknown/release/full_access_key_fallback_base.wasm`

   ```shell
   $ ./build.sh
   ```

3. **Deploy and init a contract**
   ```shell
   $ near deploy --accountId <CONTRACT_ACCOUNT> --wasmFile ./target/wasm32-unknown-unknown/release/full_access_key_fallback_base.wasm --initFunction new --initArgs '{}'
   ```

## Example of using the contract with full access key fallback plugin
Our plan for this example it is remove the full access key and after that bring it is back. 
The keys usually storage at `~/.near-credentials/testnet/<CONTRACT_ACCOUNT>.json`. Also let's
choose some operation which only the contract with full access key can done, for example, 
transfer money. 

Move ownership rights to the Alice account. 
```shell
$ near call <CONTRACT_ACCOUNT> owner_set '{"owner": "<ALICE_ACCOUNT>"}' --accountId <CONTRACT_ACCOUNT>
```

Check that currently we can transfer the money for example to Alice account
```shell
$ near send <CONTRACT_ACCOUNT> <ALICE_ACCOUNT> 1
```

Now removing the full access key for contract account

```shell
$ near delete-key <CONTRACT_ACCOUNT> "ed25519:ErVTCTvmepb4NDhQ7infTomkLVsd1iTWwLR84FBhV7UC"
```

The value of public-key can be found at `~/.near-credentials/testnet/<CONTRACT_ACCOUNT>.json`. 

Now check, that now the money transfer will not work: 
```shell
$ near send <CONTRACT_ACCOUNT> <ALICE_ACCOUNT> 1
```

And now let's add the key back and check that it will work
```shell
$ near call <CONTRACT_ACCOUNT> attach_full_access_key '{"public_key": "ed25519:ErVTCTvmepb4NDhQ7infTomkLVsd1iTWwLR84FBhV7UC"}' --accountId <ALICE_ACCOUNT>
$ near send <CONTRACT_ACCOUNT> <ALICE_ACCOUNT> 1
```

## Tests running instruction
Tests in `src/lib.rs` contain examples of interaction with a contract.

For running test:
1. Generate `wasm` file by running `build.sh` script. The target file will be `target/wasm32-unknown-unknown/release/full_access_key_fallback_base.wasm`
2. Run tests `cargo test`

```shell
$ ./build.sh
$ cargo test
```

For tests, we use `workspaces` library and `sandbox` environment for details you can explorer `../near-plugins-test-utils` crate
contract_account.