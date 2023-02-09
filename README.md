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

Basic access control mechanism that allows _only_ an authorized account id to call certain methods. Note this account id can belong either to a regular user, or it could be a contract (a DAO for example).

[This contract](/near-plugins/tests/contracts/ownable/src/lib.rs) provides an example of using `Ownable`. It is compiled, deployed on chain and interacted with in [integration tests](/near-plugins/tests/ownable.rs).

Documentation of all methods provided by the derived implementation of `Ownable` is available in the [definition of the trait](/near-plugins/src/ownable.rs).

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

Allows a contract to be upgraded by owner with delay and without having a Full Access Key.

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
        contract.up_init_staging_duration(std::time::Duration::from_secs(60).as_nanos().try_into().unwrap()); // 1 minute
        contract
    }
}
```

To upgrade the contract first call `up_stage_code` passing the binary as first argument serialized as borsh. Then call `up_deploy_code`.
This functions must be called from the owner.

To update the staging delay first call `up_stage_update_staging_duration` passing the new delay duration. Then call `up_apply_update_staging_duration`.
This functions must be called from the owner.

Documentation of all methods provided by the derived implementation of `Upgradable` is available in the [definition of the trait](/near-plugins/src/upgradable.rs). More examples and guidelines for interacting with an `Upgradable` contract can be found [here](/examples/upgradable-examples/README.md).

### [AccessControllable](/near-plugins/src/access_controllable.rs)

Enables role-based access control for contract methods. A method with restricted access can only be called _successfully_ by accounts that have been granted one of the whitelisted roles. If a restricted method is called by an account with insufficient permissions, it panics.

Each role is managed by admins who may grant the role to accounts and revoke it from them. In addition, there are super admins that have admin permissions for every role. The sets of accounts that are (super) admins and grantees are stored in the contract's state.

[This contract](/near-plugins/tests/contracts/access_controllable/src/lib.rs) provides an example of using `AccessControllable`. It is compiled, deployed on chain and interacted with in [integration tests](/near-plugins/tests/access_controllable.rs).

Documentation of all methods provided by `AccessControllable` is available in the [definition of the trait](/near-plugins/src/access_controllable.rs).

## Internal Architecture

Each plugin's functionality is described by a trait defined in `near-plugins/src/<plugin_name>.rs`. The trait's methods will be available on contracts that use the corresponding plugin, whereas the implementation of the trait is provided by procedural macros.

The code that is generated for a trait implementation is based on `near-plugins-derive/src/<plugin_name.rs>`. To inspect the code generated for your particular smart contract, [`cargo-expand`](https://github.com/dtolnay/cargo-expand) can be helpful.

## Testing

Tests should verify that once the macros provided by this crate are expanded, the contract they are used in has the intended functionality. Integration tests are utilized for that purpose:

- A contract using the plugin is contained in `near-plugins/tests/contracts/<plugin_name>/`.
- This contract is used in `near-plugins/tests/<plugin_name>.rs` which:
    - Compiles and deploys the contract on chain via [NEAR `workspaces`](https://docs.rs/workspaces/0.7.0/workspaces/).
    - Sends transactions to the deployed contract to verify plugin functionality.

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

