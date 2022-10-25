# Example of using Ownable plugin

A basic access control mechanism that allows only an authorized account ID to call certain methods. Note, this account ID can belong either to a regular user, or it could be a contract (a DAO for example).

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

### owner_set
`owner_set` is the method that sets the new owner of the contract. Only the current owner of the contract can set a new one.
If the contract doesn't have any owner, the self can set one.

```shell
$ near call <CONTRACT_ACCOUNT> owner_set '{"owner": <NEW_OWNER_ACCOUNT>}' --accountId <OWNER_ACCOUNT>
```

### owner_get
`owner_get` is the _view_ method, which returns the current contract owner.

```shell
$ near view <CONTRACT_ACCOUNT> owner_get '{}'
View call: <CONTRACT_ACCOUNT>.owner_get({})
'<OWNER_ACCOUNT>'
```

### owner_storage_key
`owner_storage_key` is the _view_ method that returns the key for storage, the owner account id.
The `__OWNER__` by default. It can be changed by using the `ownable` attribute. See example `ownable_change_storage_key`.

```shell
$ near view <CONTRACT_ACCOUNT> owner_storage_key
View call: <CONTRACT_ACCOUNT>.owner_storage_key()
[
  95, 95, 79, 87, 78,
  69, 82, 95, 95
]
$ python3
>>> print(' '.join(str(b) for b in bytes("__OWNER__", 'utf8')))
95 95 79 87 78 69 82 95 95
```

### owner_is
`owner_is` is the method that checks if the caller is the owner of the contract.

```shell
$ near call <CONTRACT_ACCOUNT> owner_is --accountId <OWNER_ACCOUNT>
Scheduling a call: <CONTRACT_ACCOUNT>.testnet.owner_is()
Doing account.functionCall()
Transaction Id <TRANSACTION_ID>
To see the transaction in the transaction explorer, please open this url in your browser
https://explorer.testnet.near.org/transactions/<TRANSACTION_ID>
true
```

## Example of using a contract with the ownable plugin
In that document, we are providing some examples of using contracts with the ownable plugin. You also can explore the usage examples in the tests in `ownable_base/src/lib.rs`. For running tests, please take a look at the **Test running instruction** section.

### Preparation steps for demonstration
1. **Creating an account on testnet**

   For demonstration, we should create a few accounts on NEAR testnet. Let's say we will create two accounts on NEAR testnet: (1) <CONTRACT_ACCOUNT_NAME>.<MASTER_ACCOUNT_NAME>.testnet, (2) <OWNER_ACCOUNT_NAME>.<MASTER_ACCOUNT_NAME>.testnet. You should pick some unique names for accounts.

   The instruction of creating an accounts on NEAR testnet: https://docs.near.org/tools/near-cli#near-create-account
   
   ```shell
   $ near create-account <CONTRACT_ACCOUNT_NAME>.<MASTER_ACCOUNT_NAME>.testnet --masterAccount <MASTER_ACCOUNT_NAME>.testnet --initialBalance 10
   $ near create-account <OWNER_ACCOUNT_NAME>.<MASTER_ACCOUNT_NAME>.testnet --masterAccount <MASTER_ACCOUNT_NAME>.testnet --initialBalance 10
   ```

   In the next sections we will refer to the `<CONTRACT_ACCOUNT_NAME>.<MASTER_ACCOUNT_NAME>.testnet` as `<CONTRACT_ACCOUNT>` and to the `<OWNER_ACCOUNT_NAME>.<MASTER_ACCOUNT_NAME>.testnet` as `<OWNER_ACCOUNT>` for simplicity. 

2. **Compile Contract to wasm file**
   For compiling the contract just run the `ownable_base/build.sh` script. The target file with compiled contract will be `../target/wasm32-unknown-unknown/release/ownable_base.wasm`
 
   ```shell
   $ cd ownable_base
   $ ./build.sh
   $ cd ..
   ```

3. **Deploy and init a contract**
   ```shell
   $ near deploy --accountId <CONTRACT_ACCOUNT> --wasmFile ../target/wasm32-unknown-unknown/release/ownable_base.wasm --initFunction new --initArgs '{}'
   ```

### Contract with owner plugin usage examples
#### When the self is the owner it can call all methods
After initialization, the `<CONTRACT_ACCOUNT>` is the owner of this contract. So this account is both the `self` and the `owner` of the contract, and it can call all the contract methods.

```shell
$ near call <CONTRACT_ACCOUNT> protected '{}' --accountId <CONTRACT_ACCOUNT>
$ near call <CONTRACT_ACCOUNT> protected_owner '{}' --accountId <CONTRACT_ACCOUNT>
$ near call <CONTRACT_ACCOUNT> protected_self '{}' --accountId <CONTRACT_ACCOUNT>
$ near call <CONTRACT_ACCOUNT> unprotected '{}' --accountId <CONTRACT_ACCOUNT>
```

We can check that we succeeded in calling all these functions by calling the `get_counter` view method and checking that the counter is 4.

```shell
$ near view <CONTRACT_ACCOUNT> get_counter '{}' 
View call: <CONTRACT_ACCOUNT>.get_counter({})
4
```

#### The stranger accounts can use only unprotected functions
Currently, the `<OWNER_ACCOUNT>` doesn't connected to the contract. So, we can check that we can only succeed in calling `unprotected` method and will fail on calling all other protected methods.

```shell
$ near call <CONTRACT_ACCOUNT> protected '{}' --accountId <OWNER_ACCOUNT>
ERROR
$ near call <CONTRACT_ACCOUNT> protected_owner '{}' --accountId <OWNER_ACCOUNT>
ERROR
$ near call <CONTRACT_ACCOUNT> protected_self '{}' --accountId <OWNER_ACCOUNT>
ERROR
$ near view <CONTRACT_ACCOUNT> get_counter '{}' 
View call: <CONTRACT_ACCOUNT>.get_counter({})
4
$ near call <CONTRACT_ACCOUNT> unprotected '{}' --accountId <OWNER_ACCOUNT>
$ near view <CONTRACT_ACCOUNT> get_counter '{}' 
View call: <CONTRACT_ACCOUNT>.get_counter({})
5
```

#### Check and Change the contract owner
Let's change the contract owner from `<CONTRACT_ACCOUNT>` to `<OWNER_ACCOUNT>`. Only the current owner of the contract can change the owner.

We can check the owner of the contract by calling `owner_get` view method.
```shell
$ near view <CONTRACT_ACCOUNT> owner_get '{}'
View call: <CONTRACT_ACCOUNT>.owner_get({})
'<CONTRACT_ACCOUNT>'
```

In this case, the owner is `<CONTRACT_ACCOUNT>`. And we can change the contract owner by running `owner_set`.
```shell
$ near call <CONTRACT_ACCOUNT> owner_set '{"owner": <OWNER_ACCOUNT>}' --accountId <CONTRACT_ACCOUNT>
```

And we can check the contract owner one more time for making sure, that it is changed.
```shell
$ near view <CONTRACT_ACCOUNT> owner_get '{}'
View call: <CONTRACT_ACCOUNT>.owner_get({})
'<OWNER_ACCOUNT>'
```

#### When the self is not owner it can't run the only(owner) functions
So, now `<CONTRACT_ACCOUNT>` is not the owner of our contract anymore. The `<CONTRACT_ACCOUNT>` can run the `unprotected`, `proteced_self`, `protected` and can't use the methods `protected_owner`.

```shell
$ near call <CONTRACT_ACCOUNT> protected '{}' --accountId <CONTRACT_ACCOUNT>
$ near call <CONTRACT_ACCOUNT> unprotected '{}' --accountId <CONTRACT_ACCOUNT>
$ near call <CONTRACT_ACCOUNT> protected_self '{}' --accountId <CONTRACT_ACCOUNT>
$ near view <CONTRACT_ACCOUNT> get_counter '{}' 
View call: <CONTRACT_ACCOUNT>.get_counter({})
8
$ near call <CONTRACT_ACCOUNT> protected_owner '{}' --accountId <CONTRACT_ACCOUNT>
ERROR
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
ERROR
$ near view <CONTRACT_ACCOUNT> get_counter '{}' 
View call: <CONTRACT_ACCOUNT>.get_counter({})
11
```
#### Only owner can change the contract ownership
When the contract has an owner, only the owner can change the ownership. All other accounts, including self, can't.

```shell
$ near view <CONTRACT_ACCOUNT> owner_get '{}'
View call: <CONTRACT_ACCOUNT>.owner_get({})
'<OWNER_ACCOUNT>'
$ near call <CONTRACT_ACCOUNT> owner_set '{"owner": <CONTRACT_ACCOUNT>}' --accountId <CONTRACT_ACCOUNT>
ERROR
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

#### The self can't run the only(owner) function if contract doesn't have an owner
When the contract doesn't have an owner, no one can use `only(owner)` functions including self.

```shell
$ near view <CONTRACT_ACCOUNT> get_counter '{}' 
View call: <CONTRACT_ACCOUNT>.get_counter({})
11
$ near call <CONTRACT_ACCOUNT> protected_owner '{}' --accountId <CONTRACT_ACCOUNT>
ERROR
$ near view <CONTRACT_ACCOUNT> get_counter '{}' 
View call: <CONTRACT_ACCOUNT>.get_counter({})
11
```

#### When the contract doesn't have owner, the self can set up a new one
When the contract doesn't have the owner, the self can set up a new owner.
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
Tests in `ownable_base/src/lib.rs` contain examples of interaction with a contract. 

For running test: 
1. Generate `wasm` file by running `build.sh` script. The target file will be `../target/wasm32-unknown-unknown/release/ownable_base.wasm`
2. Run tests `cargo test`

```shell
$ cd ownable_base
$ ./build.sh
$ cargo test
```

For tests, we use `workspaces` library and `sandbox` environment. For details, you can explore `../near-plugins-test-utils` crate.