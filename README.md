# NEAR Smart Contracts Plugins

Implementation of common patterns used for NEAR smart contracts. Macros provided by default assumes the contract is
using near-sdk-rs and `#[near_bindgen]` macro.

## Plugins

Documentation and implementation details of each plugin can be found in the source code, primarily in the [traits](/near-plugins/src/) which define plugin behavior. Events emitted by each plugin
are also described in the [source code](/near-plugins-derive/src/) of each macro. Each event follows [NEP-297](https://nomicon.io/Standards/EventsFormat).

The following sections provide an overview of all available plugins. More examples and usage patterns are available in:

- [`examples/`](/examples/)
- [`near-plugins/tests/contracts/`](/near-plugins/tests/contracts/)

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

Documentation of all methods provided by the derived implementation of `Ownable` is available in the [definition of the trait](/near-plugins/src/ownable.rs). More examples and guidelines for interacting with an `Ownable` contract can be found [here](/examples/ownable-examples/README.md).

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

Documentation of all methods provided by the derived implementation of `FullAccessKeyFallback` is available in the [definition of the trait](/near-plugins/src/full_access_key_fallback.rs). More examples and guidelines for interacting with a `FullAccessKeyFallback` contract can be found [here](/examples/full-access-key-fallback-examples/README.md).

### [Pausable](/near-plugins/src/pausable.rs)

Allow contracts to implement an emergency stop mechanism that can be triggered by an authorized account. Pauses can be
used granularly to only limit certain features.

Using the `Pausable` plugin requires the contract to be _AccessControllable_ in order to manage permissions. Roles allowing accounts to call certain methods can be granted and revoked via the _AccessControllable_ plugin.

[This contract](/near-plugins/tests/contracts/pausable/src/lib.rs) provides an example of using `Pausable`. It is compiled, deployed on chain and interacted with in [integration tests](/near-plugins/tests/pausable.rs).

Documentation of all methods provided by `Pausable` is available in the [definition of the trait](/near-plugins/src/pausable.rs).

### [Upgradable](/near-plugins/src/upgradable.rs)

Allows a contract to be upgraded by owner without having a Full Access Key.

Contract example using _Upgradable_ plugin. Note that it requires the contract to be Ownable.

```rust
#[near_bindgen]
#[derive(Ownable, Upgradable)]
struct Counter;

#[near_bindgen]
impl Counter {
    /// Specify the owner of the contract in the constructor
    #[init]
    fn new() -> Self {
        let mut contract = Self {};
        contract.owner_set(Some(near_sdk::env::predecessor_account_id()));
        contract
    }
}
```

To upgrade the contract first call `up_stage_code` passing the binary as first argument serialized as borsh. Then call `up_deploy_code`.
This functions must be called from the owner.

Documentation of all methods provided by the derived implementation of `Upgradable` is available in the [definition of the trait](/near-plugins/src/upgradable.rs). More examples and guidelines for interacting with an `Upgradable` contract can be found [here](/examples/upgradable-examples/README.md).

### [AccessControllable](/near-plugins/src/access_controllable.rs)

Enables role-based access control for contract methods. A method with restricted access can only be called _successfully_ by accounts that have been granted one of the whitelisted roles. If a restricted method is called by an account with insufficient permissions, it panics.

Each role is managed by admins who may grant the role to accounts and revoke it from them. In addition, there are super admins that have admin permissions for every role.

The sets of accounts that are (super) admins and grantees are stored in the contract's state.

```rust
/// Roles are represented by enum variants.
/// 
/// Deriving `AccessControlRole` ensures `Role` can be used in
/// `AccessControllable`.
#[derive(AccessControlRole, Deserialize, Serialize, Copy, Clone)]
#[serde(crate = "near_sdk::serde")]
pub enum Role {
    SkipperByOne,
    SkipperByAny,
    Resetter,
}

#[access_control(role_type(Role))]
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    counter: u64,
}

#[near_bindgen]
impl Counter {
    /// Setup access control in the constructor.
    #[init]
    pub fn new() -> Self {
        let contract = Self {
            counter: 0,
            // Initialize `AccessControllable` plugin state.
            __acl: Default::default(),
        };

        // Add the contract itself as super admin.
        near_sdk::require!(
            contract.acl_init_super_admin(near_sdk::env::predecessor_account_id()),
            "Failed to initialize super admin",
        );

        // Specify an account to be added as admin for a specific role.
        let skipper_by_one_admin_account_id: AccountId = "alice.near".parse().unwrap();

        // Add an admin for `Role::SkipperByOne`. This is possible since the
        // contract was just made a super admin.
        let result = contract.acl_add_admin(
            Role::SkipperByOne.into(),
            skipper_by_one_admin_account_id,
        );
        near_sdk::require!(Some(true) == result, "Failed to add admin");

        // Specify an account to be granted a specific role.
        let skipper_by_any_grantee_account_id: AccountId = "bob.near".parse().unwrap();

        // Grant a role. Also possible since the contract is super admin.
        let result = contract.acl_grant_role(
            Role::SkpperByAny.into(),
            skipper_by_any_grantee_account_id,
        );
        near_sdk::require!(Some(true) == result, "Failed to grant role");
    }

    /// This method has no access control. Anyone can call it successfully.
    pub fn increase(&mut self) {
       self.counter += 1; 
    }

    /// Only an account that was granted either `Role::SkipperByOne` or
    /// `Role::SkipperByAny` may successfully call this method.
    #[access_control_any(roles(Role::SkipperByOne, Role::SkipperByAny))]
    pub fn skip_one(&mut self) {
        self.counter += 2;
    }

    /// Only an account that was granted `Role:Resetter` may successfully call
    /// this method.
    #[access_control_any(roles(Role::Resetter))]
    pub fn reset(&mut self) {
        self.counter = 0;
    }
}
```

The derived implementation of `AccessControllable` provides more methods that are documented in the [definition of the trait](/near-plugins/src/access_controllable.rs). More usage patterns are explained in [examples](/examples/access-controllable-examples/) and in [integration tests](/near-plugins/tests/access_controllable.rs).

## Contributors Notes

Traits doesn't contain any implementation, even though some interfaces are self-contained enough to have it.
It is this way since `near_bindgen` macro from near-sdk-rs will only expose as public methods those that are implemented
during the trait implementation for the contract.

In the documentation all comments under Default Implementation makes remarks about the current implementation derived
automatically from macros. They can be changed if the trait is manually implemented rather than deriving the macro.

## Roadmap

- Factory upgrades: Allow upgrading all deployed contracts from the factory fetching binary upstream.
- Events ergonomics. `Event` macro that can be used in the following way:
```rust
#[derive(Serialize, Event(standard="nepXXX", version="1.0.1", action="transfer"))]
struct Transfer { 
    value: u64
}

/// In the contract
let transfer = Transfer { value: 1 };
transfer.emit(); // At this step the event is serialized and the log is emitted.
```
- Allow deriving plugins privately, i.e. without making the methods public.
    This will allow developers to create custom logic on top of the plugin without modifying source code.

