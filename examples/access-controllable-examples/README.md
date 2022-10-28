# Example of using the Access Control plugin

An access control mechanism that allows you to specify which groups of users can access certain functions.

```rust
use near_plugins::AccessControllable;
use near_plugins::AccessControlRole;
use near_plugins_derive::access_control;
use near_plugins_derive::access_control_any;
use near_sdk::near_bindgen;
use borsh::{BorshSerialize, BorshDeserialize};
use near_sdk::env;

/// All types of access groups
#[derive(AccessControlRole, Clone, Copy)]
pub enum UsersGroups {
   GroupA,
   GroupB,
}

#[near_bindgen]
#[access_control(role_type(UsersGroups))]
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

      contract.acl_init_super_admin(near_sdk::env::predecessor_account_id());

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
`acl_storage_prefix` is a method that returns the common prefix of keys for storing the members and the admins of groups.
`__acl` by default.

```shell
$ near call <CONTRACT_ACCOUNT> acl_storage_prefix --accountId <ALICE_ACCOUNT>
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

Example of changing acl storage prefix key:
```rust
#[near_bindgen]
#[access_control(role_type(UsersGroups), storage_prefix = "__custom_prefix")]
#[derive(Default, BorshSerialize, BorshDeserialize)]
struct Counter {
  counter: u64,
}
```

### acl_init_super_admin
`acl_init_super_admin` is a method that adds `account_id` as a super-admin _without_ checking any permissions
in case there are no super-admins. Do nothing if at least one super-admin exists. This function can be used to add a super-admin during contract initialization. 
Moreover, it may provide a recovery mechanism if (mistakenly) all super-admins have been removed. 

**Return value:** the return value indicates whether `account_id` was added as super-admin.

It is `#[private]` in the implementation provided by this trait, i.e. only the contract itself may call this method.

```shell
$ near call <CONTRACT_ACCOUNT> acl_init_super_admin '{"account_id": "<SUPER_ADMIN_ACCOUNT>"}' --accountId <CONTRACT_ACCOUNT>
true
```

If the method succeeds, the following event will be emitted:
```json
{
   "standard":"AccessControllable",
   "version":"1.0.0",
   "event":"super_admin_added",
   "data":{
      "account":"test.near",
      "by":"test.near"
   }
}
```

### acl_is_super_admin
`acl_is_super_admin` is a _view_ method that checks that account has super admin rights.
Super admin can control the member list of each group and control the admin list for each group.

```shell
$ near view <CONTRACT_ACCOUNT> acl_is_super_admin '{"account_id": "<SUPER_ADMIN_ACCOUNT>" }' 
View call: <CONTRACT_ACCOUNT>.acl_is_super_admin({"account_id": "<SUPER_ADMIN_ACCOUNT>"})
true
```

### acl_add_admin
`acl_add_admin` is a method that adds a new admin for a specific group.
Admins' rights don't allow running group-specific functions, but group admins can control the group member list.
This method can be run by an admin of a specific group or by a super admin.

**Return value:** in case of sufficient permissions, the returned `Some(bool)` indicates
whether `account_id` is a new admin for `role`. Without permissions,
`None` is returned and internal state is not modified.

```shell
$ near call <CONTRACT_ACCOUNT> acl_add_admin '{"role": "GroupA", "account_id": "<ALICE_ACCOUNT>"}' --accountId <GROUP_A_ADMIN_ACCOUNT>
true
```

If the method succeeds, the following event will be emitted:
```json
{
   "standard":"AccessControllable",
   "version":"1.0.0",
   "event":"admin_added",
   "data": {
      "role":"GroupA",
      "account":"<ALICE_ACCOUNT>",
      "by":"<GROUP_A_ADMIN_ACCOUNT>"
   }
}
```

### acl_is_admin
`acl_is_admin` is a _view_ method that checks if the account has an admin right for the specified group. For super admin, it will return true for every group.

```shell
$ near view <CONTRACT_ACCOUNT> acl_is_admin '{"role": "GroupA", "account_id": "<ALICE_ACCOUNT>"}'
View call: <CONTRACT_ACCOUNT>.acl_is_admin({"role": "GroupA", "account_id": "<ALICE_ACCOUNT>"})
true
```

### acl_revoke_admin
`acl_revoke_admin` is a method that removes the group admin right for a specific account. Can be executed by an admin of this group or by a super admin.

**Return value:** in case of sufficient permissions, the returned `Some(bool)` indicates
whether `account_id` was an admin for `role`. Without permissions,
`None` is returned and internal state is not modified.

```shell
$ near call <CONTRACT_ACCOUNT> acl_revoke_admin '{"role": "GroupA", "account_id": "<ALICE_ACCOUNT>"}' --accountId <GROUP_A_ADMIN_ACCOUNT>
true
```

If the method succeeds, the following event will be emitted:
```json
{
   "standard":"AccessControllable",
   "version":"1.0.0",
   "event":"admin_revoked",
   "data":{
      "role":"GroupA",
      "account":"<ALICE_ACCOUNT>",
      "by":"<GROUP_A_ADMIN_ACCOUNT>"
   }
}
```

### acl_renounce_admin
`acl_renounce_admin` is a method that removes the group admin right for an account that calls the method.

**Return value:** returns whether the predecessor was an admin for `role`.

```shell
$ near call <CONTRACT_ACCOUNT> acl_renounce_admin '{"role": "GroupA"}' --accountId <ALICE_ACCOUNT>
true
```

After calling that method, Alice will not have the admin right for GroupA anymore.

If the method succeeds, the following event will be emitted:
```json
{
   "standard":"AccessControllable",
   "version":"1.0.0",
   "event":"admin_revoked",
   "data":{
      "role":"GroupA",
      "account":"<ALICE_ACCOUNT>",
      "by":"<ALICE_ACCOUNT>"
   }
}
```
### acl_revoke_role
`acl_revoke_role` is a method that removes the specified account from the list of the group members.
Only a group admin or a super admin can execute this function.

**Return value:** in case of sufficient permissions, the returned `Some(bool)` indicates
whether `account_id` was a grantee of `role`. Without permissions,
`None` is returned and internal state is not modified.

```shell
$ near call <CONTRACT_ACCOUNT> acl_revoke_role '{"role": "GroupA", "account_id": "<ALICE_ACCOUNT>"}' --accountId <GROUP_A_ADMIN_ACCOUNT>
true
```

If the method succeeds, the following event will be emitted:
```json
{
   "standard":"AccessControllable",
   "version":"1.0.0",
   "event":"role_revoked",
   "data": {
      "role":"GroupA",
      "from":"<ALICE_ACCOUNT>",
      "by":"<GROUP_A_ADMIN_ACCOUNT>"
   }
}
```

### acl_renounce_role
`acl_renounce_role` is a method that removes the caller account from the member list of the group. Can be called by anyone.

**Return value:** returns whether it was a grantee of `role`.

```shell
$ near call <CONTRACT_ACCOUNT> acl_renounce_role '{"role": "GroupA"}' --accountId <ALICE_ACCOUNT>
true
```

If the method succeeds, the following event will be emitted:
```json
{
   "standard":"AccessControllable",
   "version":"1.0.0",
   "event":"role_revoked",
   "data": {
      "role":"GroupA",
      "from":"<ALICE_ACCOUNT>",
      "by":"<ALICE_ACCOUNT>"
   }
}
```
### acl_grant_role
`acl_grant_role` is a method that adds the account to the group member list. Can be executed only by a group admin or by a super admin.

**Return value:** in case of sufficient permissions, the returned `Some(bool)` indicates
whether `account_id` is a new grantee of `role`. Without permissions,
`None` is returned and internal state is not modified.

```shell
$ near call <CONTRACT_ACCOUNT> acl_grant_role '{"role": "GroupA", "account_id": "<ALICE_ACCOUNT>"}' --accountId <GROUP_A_ADMIN_ACCOUNT>
true
```

If the method succeeds, the following event will be emitted:
```json
{
   "standard":"AccessControllable",
   "version":"1.0.0",
   "event":"role_granted",
   "data": {
      "role":"GroupA",
      "to":"<ALICE_ACCOUNT>",
      "by":"<GROUP_A_ADMIN_ACCOUNT>"
   }
}
```

### acl_has_role
`acl_has_role` is a _view_ method for checking if the account is a member of the specified group.

```shell
$ near view <CONTRACT_ACCOUNT> acl_has_role '{"role": "GroupA", "account_id": "<ALICE_ACCOUNT>"}'
View call: <CONTRACT_ACCOUNT>.acl_has_role({"role": "GroupA", "account_id": "<ALICE_ACCOUNT>"})
true
```

### acl_has_any_role
`acl_has_any_role` is a _view_ method for checking if an account is a member of at least one of the specified groups.

```shell
$ near view <CONTRACT_ACCOUNT> acl_has_any_role '{"roles": ["GroupA", "GroupB"], "account_id": "<ALICE_ACCOUNT>"}'
View call: <CONTRACT_ACCOUNT>.acl_has_any_role({"roles": ["GroupA", "GroupB"], "account_id": "<ALICE_ACCOUNT>"})
true
```

### acl_get_super_admins
`acl_get_super_admins` is a _view_ method that shows some super admins. It will skip first `skip` admins and return not more than `limit` number of super admins.

```shell
$ near view <CONTRACT_ACCOUNT> acl_get_super_admins '{"skip": 0, "limit": 2}'
View call: <CONTRACT_ACCOUNT>.acl_get_super_admins({"skip": 0, "limit": 2})
[ '<CONTRACT_ACCOUNT>' ]
```

### acl_get_admins
`acl_get_admins` is a _view_ method that shows some admins of the group. It will skip first `skip` admins and return not more than `limit` number of admins.

```shell
$ near view <CONTRACT_ACCOUNT> acl_get_admins '{"role": "GroupA", "skip": 0, "limit": 2}'
View call: <CONTRACT_ACCOUNT>.acl_get_admins({"role": "GroupA", "skip": 0, "limit": 2})
[ '<ALICE_ACCOUNT>' ]
```

### acl_get_grantees
`acl_get_grantess` is a _view_ method that shows some members of the group. It will skip the first `skip` members and return not more than  `limit` number of members.

```shell
$ near view <CONTRACT_ACCOUNT> acl_get_grantess '{"role": "GroupA", "skip": 0, "limit": 2}'
View call: <CONTRACT_ACCOUNT>.acl_get_grantess({"role": "GroupA", "skip": 0, "limit": 2})
[ '<ALICE_ACCOUNT>' ]
```

## Preparation steps for demonstration
In that document, we are providing some examples of using a contract with an access control plugin. You also can explore the usage examples in the tests in `./access_controllable_base/src/lib.rs` and in `./access_control_role_base/src/lib.rs`. For running tests, please take a look at the **Test running instruction** section.

1. **Creating an account on testnet**
   For demonstration let's create 3 accounts: `<CONTRACT_ACCOUNT>`, `<ALICE_ACCOUNT>`, `<BOB_ACCOUNT>`
   ```shell
   $ near create-account <CONTRACT_ACCOUNT_NAME>.<MASTER_ACCOUNT_NAME>.testnet --masterAccount <MASTER_ACCOUNT_NAME>.testnet --initialBalance 10
   $ near create-account <ALICE_ACCOUNT_NAME>.<MASTER_ACCOUNT_NAME>.testnet --masterAccount <MASTER_ACCOUNT_NAME>.testnet --initialBalance 10
   $ near create-account <BOB_ACCOUNT_NAME>.<MASTER_ACCOUNT_NAME>.testnet --masterAccount <MASTER_ACCOUNT_NAME>.testnet --initialBalance 10
   ```

   In the next sections, we will refer to the `<CONTRACT_ACCOUNT_NAME>.<MASTER_ACCOUNT_NAME>.testnet` as `<CONTRACT_ACCOUNT>`, 
   to the `<ALICE_ACCOUNT_NAME>.<MASTER_ACCOUNT_NAME>.testnet` as `<ALICE_ACCOUNT>`, and to the `<BOB_ACCOUNT_NAME>.<MASTER_ACCOUNT_NAME>.testnet` as `<BOB_ACCOUNT>` for simplicity.

2. **Compile Contract to wasm file**
   For compiling the contract, just run the `access_controllable_base/build.sh` script. The target file with compiled contract will be `../target/wasm32-unknown-unknown/release/access_controllable_base.wasm`

   ```shell
   $ cd access_controllable_base
   $ ./build.sh
   $ cd ..
   ```

3. **Deploy and init a contract**
   ```shell
   $ near deploy --accountId <CONTRACT_ACCOUNT> --wasmFile ../target/wasm32-unknown-unknown/release/access_controllable_base.wasm --initFunction new --initArgs '{}'
   ```

## Example of using the contract with access control plugin
### Calling the methods with access control
For using the method, `level_a_incr` you should be a member of GroupA.  Alice is not a member of any group, so she can't use this method.

```shell
$ near call <CONTRACT_ACCOUNT> level_a_incr --accountId <ALICE_ACCOUNT>
ERROR
$ near view get_counter
0
```

Let's make Alice a member of GroupA.
```shell
$ near call <CONTRACT_ACCOUNT> acl_grant_role '{"role": "GroupA", "account_id": "<ALICE_ACCOUNT>"}' --accountId <CONTRACT_ACCOUNT>
```

Now Alice is a member of GroupA and can call the `level_a_incr` method
```shell
$ near call <CONTRACT_ACCOUNT> level_a_incr --accountId <ALICE_ACCOUNT>
$ near view get_counter
1
```

As well as calls the `level_ab_incr` method, which allowed for both GroupA and GroupB members.
```shell
$ near call <CONTRACT_ACCOUNT> level_ab_incr --accountId <ALICE_ACCOUNT>
$ near view get_counter
2
```

### Admin of the group not a member of the group
Note, the admin of the group may not be a member of this group. For example, the `<CONTRACT_ACCOUNT>` is a super admin, but he
can't execute the `level_a_incr` method. 

```shell
$ near view get_counter
2
$ near call <CONTRACT_ACCOUNT> level_a_incr --accountId <CONTRACT_ACCOUNT>
ERROR
$ near view get_counter
2
```

## Tests running instruction
Tests in `access_controllable_base/src/lib.rs` contain examples of interaction with a contract.

For running test:
1. Generate `wasm` file by running `access_controllable_base/build.sh` script. The target file will be `../target/wasm32-unknown-unknown/release/access_controllable_base.wasm`
2. Run tests `cargo test`

```shell
$ cd access_controllable_base
$ ./build.sh
$ cargo test
```

For tests, we use `workspaces` library and `sandbox` environment. For details, you can explore `../near-plugins-test-utils` crate.