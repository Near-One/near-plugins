# Example of using Access Control plugin

An access control mechanism that allows you to specify which groups of users can have access to certain functions.

```rust
use near_plugins::AccessControllable;
use near_plugins::AccessControlRole;
use near_plugins_derive::access_control;
use near_plugins_derive::access_control_any;
use near_sdk::near_bindgen;
use borsh::{BorshSerialize, BorshDeserialize};
use near_plugins::events::AsEvent;
use near_sdk::env;

/// All types of access groups
#[derive(AccessControlRole, Clone, Copy)]
pub enum UsersGroups {
    GroupA,
    GroupB,
}

#[near_bindgen]
#[access_control(role_type="UsersGroups")]
#[derive(Default, BorshSerialize, BorshDeserialize)]
struct Counter {
    counter: u64,
}

#[near_bindgen]
impl Counter {
    /// In the constructor we set up a super admin, 
    /// which can control the member lists of all user groups
    #[init]
    pub fn new() -> Self {
        let mut contract: Counter = Self{
            counter: 0,
            __acl: __Acl::default(),
        };

        contract.__acl.init_super_admin(&near_sdk::env::predecessor_account_id());

        contract
    }

    /// unprotected function, every one can call this function
    pub fn unprotected(&mut self) {
        self.counter += 1;
    }

    /// only the users from GroupA can call this method
    #[access_control_any(roles(UsersGroups::GroupA))]
    pub fn level_a_incr(&mut self) {
        self.counter += 1;
    }

    /// only the users from GroupA or GroupB can call this method
    #[access_control_any(roles(UsersGroups::GroupA, UsersGroups::GroupB))]
    pub fn level_ab_incr(&mut self) {
        self.counter += 1;
    }


    /// view method for get current counter value, every one can use it
    pub fn get_counter(&self) -> u64 {
        self.counter
    }
}
```

## The contract methods description
### acl_storage_prefix
`acl_storage_prefix` - a method which show the common prefix of keys for storage the members and the admins of groups. 
`__acl` by default. 

```shell
$ near call <CONTRACT_ACCOUNT> acl_storage_prefix --accountId acl.olga24912_3.testnet
Scheduling a call: <CONTRACT_ACCOUNT>.acl_storage_prefix()
Doing account.functionCall()
Transaction Id <TRANSACTION_ID>
To see the transaction in the transaction explorer, please open this url in your browser
https://explorer.testnet.near.org/transactions/<TRANSACTION_ID>
[ 95, 95, 97, 99, 108 ]
$ python3
>>> print(' '.join(str(b) for b in bytes("__acl", 'utf8')))
95 95 97 99 108
```

### acl_is_super_admin
`acl_is_super_admin` - a _view_ method which checks that account have a super admin rights. 
Super admin can control the members list of each group and control the admins list for each group.

```shell
$ near view <CONTRACT_ACCOUNT> acl_is_super_admin '{"account_id": "<CONTRACT_ACCOUNT>" }' 
View call: <CONTRACT_ACCOUNT>.acl_is_super_admin({"account_id": "<CONTRACT_ACCOUNT>"})
true
```

### acl_add_admin
`acl_add_admin` - add a new admin for a specific group. 
Admins right doesn't aloud to run group specific functions, but group admins can control the group member list.
This method can be run by the admin of specific group or by the super admin. 

```shell
$ near call <CONTRACT_ACCOUNT> acl_add_admin '{"role": "GroupA", "account_id": "<ALICE_ACCOUNT>"}' --accountId <CONTRACT_ACCOUNT>
```

### acl_is_admin
`acl_is_admin` is a _view_ method which checks if the account have an admin right for specified group. For super admin it will return true.

```shell
$ near view <CONTRACT_ACCOUNT> acl_is_admin '{"role": "GroupA", "account_id": "<ALICE_ACCOUNT>"}'
View call: <CONTRACT_ACCOUNT>.acl_is_admin({"role": "GroupA", "account_id": "<ALICE_ACCOUNT>"})
true
```

### acl_revoke_admin
`acl_revoke_admin` - remove the group admin right for specific account. Can be executed by admin of this group or by super admin.

```shell
$ near call <CONTRACT_ACCOUNT> acl_revoke_admin '{"role": "GroupA", "account_id": "<ALICE_ACCOUNT>"}' --accountId <CONTRACT_ACCOUNT>
```

### acl_renounce_admin
`acl_renounce_admin` - remove the group admin right for called account. 

```shell
$ near call <CONTRACT_ACCOUNT> acl_renounce_admin '{"role": "GroupA"}' --accountId <ALICE_ACCOUNT>
```

After calling that method Alice will not have the admin right for GroupA anymore. 

### acl_revoke_role
`acl_revoke_role` - remove the specified account from the list of the group member. 
Only the group admin or super admin can execute this function.

```shell
$ near call <CONTRACT_ACCOUNT> acl_revoke_role '{"role": "GroupA", "account_id": "<ALICE_ACCOUNT>"}' --accountId <CONTRACT_ACCOUNT>
```

### acl_renounce_role
`acl_renounce_role` - remove the caller account from the member list of the group. Can be called by anyone.

```shell
$ near call <CONTRACT_ACCOUNT> acl_renounce_role '{"role": "GroupA"}' --accountId <ALICE_ACCOUNT>
```

### acl_grant_role
`acl_grant_role` - add the account to the group member list. Can be executed only by the group admin or by super admin.

```shell
$ near call <CONTRACT_ACCOUNT> acl_grant_role '{"role": "GroupA", "account_id": "<ALICE_ACCOUNT>"}' --accountId <CONTRACT_ACCOUNT>
```

### acl_has_role
`acl_has_role` - a _view_ method for check if the account is a member of specified group.

```shell
$ near view <CONTRACT_ACCOUNT> acl_has_role '{"role": "GroupA", "account_id": "<ALICE_ACCOUNT>"}'
View call: <CONTRACT_ACCOUNT>.acl_has_role({"role": "GroupA", "account_id": "<ALICE_ACCOUNT>"})
true
```

### acl_has_any_role
`acl_has_any_role` - a _view_ method to check if an account a member of at least one of specified groups.

```shell
$ near view <CONTRACT_ACCOUNT> acl_has_any_role '{"roles": ["GroupA", "GroupB"], "account_id": "<ALICE_ACCOUNT>"}'
View call: <CONTRACT_ACCOUNT>.acl_has_any_role({"roles": ["GroupA", "GroupB"], "account_id": "<ALICE_ACCOUNT>"})
true
```

### acl_get_admins
`acl_get_admins` - a _view_ method which shows some group admins. It will skip first `skip` admins and return maximum `limit` number of admins.

```shell
$ near view <CONTRACT_ACCOUNT> acl_get_admins '{"role": "GroupA", "skip": 0, "limit": 2}'
View call: <CONTRACT_ACCOUNT>.acl_get_admins({"role": "GroupA", "skip": 0, "limit": 2})
[ '<ALICE_ACCOUNT>' ]
```

### acl_get_grantees
`acl_get_grantess` - a _view_ method which shows some members of the group. It will skip first `skip` members and return maximum `limit` number of members.

```shell
$ near view <CONTRACT_ACCOUNT> acl_get_grantess '{"role": "GroupA", "skip": 0, "limit": 2}'
View call: <CONTRACT_ACCOUNT>.acl_get_grantess({"role": "GroupA", "skip": 0, "limit": 2})
[ '<ALICE_ACCOUNT>' ]
```

## Preparation steps for demonstration
In that document we are providing some example of using contract with access control plugin. You also can explore the usage examples in the tests in `./src/lib.rs`. For running a tests please take a look to the **Test running instruction** section.

1. **Creating an account on testnet**
   For demonstration let's create 3 accounts: `<CONTRACT_ACCOUNT>`, `<ALICE_ACCOUNT>`, `<BOB_ACCOUNT>`
   ```shell
   $ near create-account <CONTRACT_ACCOUNT_NAME>.<MASTER_ACCOUNT_NAME>.testnet --masterAccount <MASTER_ACCOUNT_NAME>.testnet --initialBalance 10
   $ near create-account <ALICE_ACCOUNT_NAME>.<MASTER_ACCOUNT_NAME>.testnet --masterAccount <MASTER_ACCOUNT_NAME>.testnet --initialBalance 10
   $ near create-account <BOB_ACCOUNT_NAME>.<MASTER_ACCOUNT_NAME>.testnet --masterAccount <MASTER_ACCOUNT_NAME>.testnet --initialBalance 10
   ```

   In the next section we will refer to the `<CONTRACT_ACCOUNT_NAME>.<MASTER_ACCOUNT_NAME>.testnet` as `<CONTRACT_ACCOUNT>`, 
   to the `<ALICE_ACCOUNT_NAME>.<MASTER_ACCOUNT_NAME>.testnet` as `<ALICE_ACCOUNT>`, and to the `<BOB_ACCOUNT_NAME>.<MASTER_ACCOUNT_NAME>.testnet` as `<BOB_ACCOUNT>` for simplicity.

2. **Compile Contract to wasm file**
   For compiling the contract just run the `build.sh` script. The target file with compiled contract will be `./target/wasm32-unknown-unknown/release/access_controllable_base.wasm`

   ```shell
   $ ./build.sh
   ```

3. **Deploy and init a contract**
   ```shell
   $ near deploy --accountId <CONTRACT_ACCOUNT> --wasmFile ./target/wasm32-unknown-unknown/release/access_controllable_base.wasm --initFunction new --initArgs '{}'
   ```

## Example of using the contract with access control plugin
### Calling the access control methods
For using the method `level_a_incr` you should be a memeber of the GroupA.  Alice not a member of any group, so she cann't use this method.

```shell
$ near call <CONTRACT_ACCOUNT> level_a_incr --accountId <ALICE_ACCOUNT>
$ near view get_counter
0
```

Let's make the Alice the member of the GroupA. 
```shell
$ near call <CONTRACT_ACCOUNT> acl_grant_role '{"role": "GroupA", "account_id": "<ALICE_ACCOUNT>"}' --accountId <CONTRACT_ACCOUNT>
```

Now Alice the member of GroupA and call the level_a_incr method
```shell
$ near call <CONTRACT_ACCOUNT> level_a_incr --accountId <ALICE_ACCOUNT>
$ near view get_counter
1
```

As well as `level_ab_incr` which aloud for both GroupA and GroupB members.
```shell
$ near call <CONTRACT_ACCOUNT> level_ab_incr --accountId <ALICE_ACCOUNT>
$ near view get_counter
2
```

### Admin of the group not a member of the group
Note the admin of the group may not be a member of this group. For example, the `<CONTRACT_ACCOUNT>` is a super admin, but he
cann't execute the `level_a_incr` method. 

```shell
$ near view get_counter
2
$ near call <CONTRACT_ACCOUNT> level_a_incr --accountId <CONTRACT_ACCOUNT>
$ near view get_counter
2
```

## Tests running instruction
Tests in `src/lib.rs` contain examples of interaction with a contract.

For running test:
1. Generate `wasm` file by running `build.sh` script. The target file will be `target/wasm32-unknown-unknown/release/access_controllable_base.wasm`
2. Run tests `cargo test`

```shell
$ ./build.sh
$ cargo test
```

For tests, we use `workspaces` library and `sandbox` environment for details you can explorer `../near-plugins-test-utils` crate
contract_account.