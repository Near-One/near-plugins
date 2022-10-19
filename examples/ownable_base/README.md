# Example using Ownable plugin

Basic access control mechanism that allows only an authorized account id to call certain methods. Note this account id can belong either to a regular user, or it could be a contract (a DAO for example).

```Rust
use near_plugins::Ownable;
use near_sdk::near_bindgen;
use near_plugins_derive::only;
use borsh::{BorshSerialize, BorshDeserialize};

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
## The contract methods description


## Example of using contract with ownable plugin
In that document we are providing some example of using contract with ownable plugin. You also can explore the usage examples in the tests in `./src/lib.rs`. For running a tests please take a look to the **Test running instruction** section.

### Preparation steps for demonstration
1. **Creating an account on testnet**

   For demonstration, we should create a few accounts on NEAR testnet. Let's say we will create two accounts on NEAR testnet: (1) <CONTRACT_ACCOUNT_NAME>.<MASTER_ACCOUNT_NAME>.testnet, (2) <OWNER_ACCOUNT_NAME>.<MASTER_ACCOUNT_NAME>.testnet. You should pick some unique names for accounts.

   The instruction of creating an accounts on NEAR testnet: https://docs.near.org/tools/near-cli#near-create-account
   
   ```shell
   $ near create-account <CONTRACT_ACCOUNT_NAME>.<MASTER_ACCOUNT_NAME>.testnet --masterAccount <MASTER_ACCOUNT_NAME> --initialBalance 10
   $ near create-account <OWNER_ACCOUNT_NAME>.<MASTER_ACCOUNT_NAME>.testnet --masterAccount <MASTER_ACCOUNT_NAME> --initialBalance 10
   ```

   In the next section we will refer to the `<CONTRACT_ACCOUNT_NAME>.<MASTER_ACCOUNT_NAME>.testnet` as `<CONTRACT_ACCOUNT>` and to the `<OWNER_ACCOUNT_NAME>.<MASTER_ACCOUNT_NAME>.testnet` as `<OWNER_ACCOUNT>` for simplicity. 

2. **Compile Contract to wasm file**
   For compiling the contract just run the `build.sh` script. The target file with compiled contract will be `./target/wasm32-unknown-unknown/release/ownable_base.wasm`
 
   ```shell
   $ ./build.sh
   ```

3. **Deploy and init a contract**
   ```shell
   $ near deploy --accountId <CONTRACT_ACCOUNT> --wasmFile ./target/wasm32-unknown-unknown/release/ownable_base.wasm --initFunction new --initArgs '{}'
   ```

### Contract with owner plugin usage examples
#### When self is owner we can call all methods
After initialization the `<CONTRACT_ACCOUNT>` is the owner of this contract. So this account both the `self` and the `owner` of the contract and can call of the contract method. 

```shell
$ near call <CONTRACT_ACCOUNT> protected '{}' --accountId <CONTRACT_ACCOUNT>
$ near call <CONTRACT_ACCOUNT> protected_owner '{}' --accountId <CONTRACT_ACCOUNT>
$ near call <CONTRACT_ACCOUNT> protected_self '{}' --accountId <CONTRACT_ACCOUNT>
$ near call <CONTRACT_ACCOUNT> unprotected '{}' --accountId <CONTRACT_ACCOUNT>
```

We can check that we succeeded in calling all this function by calling `get_counter` view method and check that counter is 4.

```shell
$ near view <CONTRACT_ACCOUNT> get_counter '{}' 
View call: <CONTRACT_ACCOUNT>.get_counter({})
4
```

#### The stranger accounts can use only unprotected functions
Currently, the `<OWNER_ACCOUNT>` doesn't connected to the contract. So, we can check that we can only succeed in calling `unprotected` method and will fail on calling all other protected methods.

```shell
$ near call <CONTRACT_ACCOUNT> protected '{}' --accountId <OWNER_ACCOUNT>
$ near call <CONTRACT_ACCOUNT> protected_owner '{}' --accountId <OWNER_ACCOUNT>
$ near call <CONTRACT_ACCOUNT> protected_self '{}' --accountId <OWNER_ACCOUNT>
$ near view <CONTRACT_ACCOUNT> get_counter '{}' 
View call: <CONTRACT_ACCOUNT>.get_counter({})
4
$ near call <CONTRACT_ACCOUNT> unprotected '{}' --accountId <OWNER_ACCOUNT>
$ near view <CONTRACT_ACCOUNT> get_counter '{}' 
View call: <CONTRACT_ACCOUNT>.get_counter({})
5
```

#### Check and Change the contract owner
Let's change the contract owner from `<CONTRACT_ACCOUNT>` to the `<OWNER_ACCOUNT>`. Only the current owner of the contract can change the owner. 

We can check the owner of the contract by callint `owner_get` view method.
```shell
$ near view <CONTRACT_ACCOUNT> owner_get '{}'
View call: <CONTRACT_ACCOUNT>.owner_get({})
'<CONTRACT_ACCOUNT>'
```

In this case the owner is `<CONTRACT_ACCOUNT>`. And we can change the contract owner by running `owner_set`.
```shell
$ near call <CONTRACT_ACCOUNT> owner_set '{"owner": <OWNER_ACCOUNT>}' --accountId <CONTRACT_ACCOUNT>
```

And we can chack the contract owner one more time for making sure, that it is changed. 
```shell
$ near view <CONTRACT_ACCOUNT> owner_get '{}'
View call: <CONTRACT_ACCOUNT>.owner_get({})
'<OWNER_ACCOUNT>'
```

#### When self is not owner it can't run the only(owner) functions
So, now `<CONTRACT_ACCOUNT>` is not an owner of out contract anymore. So, the `<CONTRACT_ACCOUNT>` can run the `unprotected`, `proteced_self`, `protected` and can't use the methods `protected_owner`.

```shell
$ near call <CONTRACT_ACCOUNT> protected '{}' --accountId <CONTRACT_ACCOUNT>
$ near call <CONTRACT_ACCOUNT> unprotected '{}' --accountId <CONTRACT_ACCOUNT>
$ near call <CONTRACT_ACCOUNT> protected_self '{}' --accountId <CONTRACT_ACCOUNT>
$ near view <CONTRACT_ACCOUNT> get_counter '{}' 
View call: <CONTRACT_ACCOUNT>.get_counter({})
8
$ near call <CONTRACT_ACCOUNT> protected_owner '{}' --accountId <CONTRACT_ACCOUNT>
$ near view <CONTRACT_ACCOUNT> get_counter '{}' 
View call: <CONTRACT_ACCOUNT>.get_counter({})
8
```

#### Owner can't run the only(self) functions
And the owner of the contract(`<OWNER_ACCOUNT>`) can use the functions `protected`, `protected_owner` and `unprotected` and can not run the `protected_self` method.

```shell
$ near call <CONTRACT_ACCOUNT> protected '{}' --accountId <OWNER_ACCOUNT>
$ near call <CONTRACT_ACCOUNT> unprotected '{}' --accountId <OWNER_ACCOUNT>
$ near call <CONTRACT_ACCOUNT> protected_owner '{}' --accountId <OWNER_ACCOUNT>
$ near view <CONTRACT_ACCOUNT> get_counter '{}' 
View call: <CONTRACT_ACCOUNT>.get_counter({})
11
$ near call <CONTRACT_ACCOUNT> protected_self '{}' --accountId <OWNER_ACCOUNT>
$ near view <CONTRACT_ACCOUNT> get_counter '{}' 
View call: <CONTRACT_ACCOUNT>.get_counter({})
11
```
#### Only owner can change the contract ownership
When the contract have an owner only the owner can change the ownership. All other account, include self, cann't.

```shell
$ near view <CONTRACT_ACCOUNT> owner_get '{}'
View call: <CONTRACT_ACCOUNT>.owner_get({})
'<OWNER_ACCOUNT>'
$ near call <CONTRACT_ACCOUNT> owner_set '{"owner": <CONTRACT_ACCOUNT>}' --accountId <CONTRACT_ACCOUNT>
$ near view <CONTRACT_ACCOUNT> owner_get '{}'
View call: <CONTRACT_ACCOUNT>.owner_get({})
'<OWNER_ACCOUNT>'

```

#### Removing the owner of contract
We can remove the owner of the contract by set owner Null
```shell
$ near call <CONTRACT_ACCOUNT> owner_set '{"owner": null}' --accountId <OWNER_ACCOUNT>
$ near view <CONTRACT_ACCOUNT> owner_get '{}'
View call: <CONTRACT_ACCOUNT>.owner_get({})
null
```

#### The self cann't run the only(owner) function if contract doesn't have an owner
When contract doesn't have an owner no one can use only(owner) functions include self.

```shell
$ near view <CONTRACT_ACCOUNT> get_counter '{}' 
View call: <CONTRACT_ACCOUNT>.get_counter({})
11
$ near call <CONTRACT_ACCOUNT> protected_owner '{}' --accountId <CONTRACT_ACCOUNT>
$ near view <CONTRACT_ACCOUNT> get_counter '{}' 
View call: <CONTRACT_ACCOUNT>.get_counter({})
11
```

#### When the contract doesn't have owner the self can setup a new one
When the contract doesn't have the owner, the self can setup a new owner.
```shell
$ near view <CONTRACT_ACCOUNT> owner_get '{}'
View call: <CONTRACT_ACCOUNT>.owner_get({})
null
$ near call <CONTRACT_ACCOUNT> owner_set '{"owner": <OWNER_ACCOUNT>}' --accountId <CONTRACT_ACCOUNT>
$ near view <CONTRACT_ACCOUNT> owner_get '{}'
View call: <CONTRACT_ACCOUNT>.owner_get({})
'<OWNER_ACCOUNT>'
```

### Tests running instruction
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