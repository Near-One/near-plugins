# Example using Ownable plugin

Basic access control mechanism that allows only an authorized account id to call certain methods. Note this account id can belong either to a regular user, or it could be a contract (a DAO for example).

```Rust
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

    /// View method returns the value of the counter. Everyone can call it
    pub fn get_counter(&self) -> u64 {
        self.counter
    }
}
```

## Example of using contract with ownable plugin
In that document we are providing some example of using contract with ownable plugin. You also can explore the usage examples in the tests in `./src/lib.rs`. For running a tests please take a look to the **Test running instruction** section.

### Preparation steps for demonstration
1. **Creating an account on testnet**

   For demonstration, we should create a few accounts on NEAR testnet. Let's say we will create two accounts on NEAR testnet: (1) <CONTRACT_ACCOUNT>.<MASTER_ACCOUNT>.testnet, (2) <OWNER_ACCOUNT>.<MASTER_ACCOUNT>.testnet. You should pick some unique names for accounts.

   The instruction of creating an accounts on NEAR testnet: https://docs.near.org/tools/near-cli#near-create-account
   
   ```shell
   $ near create-account <CONTRACT_ACCOUNT>.<MASTER_ACCOUNT>.testnet --masterAccount <MASTER_ACCOUNT> --initialBalance 10
   $ near create-account <OWNER_ACCOUNT>.<MASTER_ACCOUNT>.testnet --masterAccount <MASTER_ACCOUNT> --initialBalance 10
   ```

2. **Compile Contract to wasm file**
   For compiling the contract just run the `build.sh` script. The target file with compiled contract will be `./target/wasm32-unknown-unknown/release/ownable_base.wasm`
 
   ```shell
   $ ./build.sh
   ```

3. **Deploy and init a contract**
   ```shell
   $ near deploy --accountId <CONTRACT_ACCOUNT>.<MASTER_ACCOUNT>.testnet --wasmFile ./target/wasm32-unknown-unknown/release/ownable_base.wasm --initFunction new --initArgs '{}'
   ```



### Test running instruction
Tests in `src/lib.rs` contain examples of interaction with a contract. 

For running test: 
1. Generate `wasm` file by running `build.sh` script. The target file will be `target/wasm32-unknown-unknown/release/ownable_base.wasm`
2. Run tests `cargo test`

```shell
$ ./build.sh
$ cargo test
```

For tests, we use `workspaces` library and `sandbox` environment for details you can explorer `../near-plugins-test-utils` crate
contract_account.