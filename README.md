# NEAR Smart Contracts Plugins

Implementation of common patterns used for NEAR smart contracts. Macros provided by default assumes the contract is
using near-sdk-rs and `#[near]` macro.

## Plugins

Documentation and implementation details of each plugin can be found in the source code, primarily in the [traits](/near-plugins/src/) which define plugin behavior. Events emitted by each plugin
are also described in the [source code](/near-plugins-derive/src/) of each macro. Each event follows [NEP-297](https://nomicon.io/Standards/EventsFormat).

The following sections provide an overview of all available plugins. More examples and usage patterns are available in [tests](/near-plugins-derive/tests) and [demo contracts](/near-plugins-derive/tests/contracts/).

### [Ownable](/near-plugins/src/ownable.rs)

Basic access control mechanism that allows _only_ an authorized account id to call certain methods. Note this account id can belong either to a regular user, or it could be a contract (a DAO for example).

[This contract](/near-plugins-derive/tests/contracts/ownable/src/lib.rs) provides an example of using `Ownable`. It is compiled, deployed on chain and interacted with in [integration tests](/near-plugins-derive/tests/ownable.rs).

Documentation of all methods provided by the derived implementation of `Ownable` is available in the [definition of the trait](/near-plugins/src/ownable.rs).

### [Pausable](/near-plugins/src/pausable.rs)

Allow contracts to implement an emergency stop mechanism that can be triggered by an authorized account. Pauses can be
used granularly to only limit certain features.

Using the `Pausable` plugin requires the contract to be _AccessControllable_ in order to manage permissions. Roles allowing accounts to call certain methods can be granted and revoked via the _AccessControllable_ plugin.

[This contract](/near-plugins-derive/tests/contracts/pausable/src/lib.rs) provides an example of using `Pausable`. It is compiled, deployed on chain and interacted with in [integration tests](/near-plugins-derive/tests/pausable.rs).

Documentation of all methods provided by `Pausable` is available in the [definition of the trait](/near-plugins/src/pausable.rs).

### [Upgradable](/near-plugins/src/upgradable.rs)

Allows a contract to be upgraded without requiring a full access key. Optionally a staging duration can be set, which defines the minimum duration that must pass before staged code can be deployed. The staging duration is a safety mechanism to protect users that interact with the contract, giving them time to opt-out before an unfavorable update is deployed.

Using the `Upgradable` plugin requires a contract to be `AccessControllable` to handle authorization for calling `Upgradable` methods to stage or deploy updates (listed below). 

To upgrade the contract, first call `up_stage_code` passing the binary as first argument serialized as borsh. Then call `up_deploy_code`.

To set a staging duration, call `up_init_staging_duration`. After initialization the staging duration can be updated by calling `up_stage_update_staging_duration` followed by `up_apply_update_staging_duration`. Updating the staging duration is itself subject to a delay: at least the currently set staging duration must pass before a staged update can be applied.

[This contract](/near-plugins-derive/tests/contracts/upgradable/src/lib.rs) provides an example of using `Upgradable`. It is compiled, deployed on chain and interacted with in [integration tests](/near-plugins-derive/tests/upgradable.rs).

Documentation of all methods provided by `Upgradable` is available in the [definition of the trait](/near-plugins/src/upgradable.rs).

### [AccessControllable](/near-plugins/src/access_controllable.rs)

Enables role-based access control for contract methods. A method with restricted access can only be called _successfully_ by accounts that have been granted one of the whitelisted roles. If a restricted method is called by an account with insufficient permissions, it panics.

Each role is managed by admins who may grant the role to accounts and revoke it from them. In addition, there are super admins that have admin permissions for every role. The sets of accounts that are (super) admins and grantees are stored in the contract's state.

[This contract](/near-plugins-derive/tests/contracts/access_controllable/src/lib.rs) provides an example of using `AccessControllable`. It is compiled, deployed on chain and interacted with in [integration tests](/near-plugins-derive/tests/access_controllable.rs).

Documentation of all methods provided by `AccessControllable` is available in the [definition of the trait](/near-plugins/src/access_controllable.rs).

## Internal Architecture

Each plugin's functionality is described by a trait defined in `near-plugins/src/<plugin_name>.rs`. The trait's methods will be available on contracts that use the corresponding plugin, whereas the implementation of the trait is provided by procedural macros.

The code that is generated for a trait implementation is based on `near-plugins-derive/src/<plugin_name.rs>`. To inspect the code generated for your particular smart contract, [`cargo-expand`](https://github.com/dtolnay/cargo-expand) can be helpful.

## Testing

Tests should verify that once the macros provided by this crate are expanded, the contract they are used in has the intended functionality. Integration tests are utilized for that purpose:

- A contract using the plugin is contained in `near-plugins-derive/tests/contracts/<plugin_name>/`.
- This contract is used in `near-plugins-derive/tests/<plugin_name>.rs` which:
    - Compiles and deploys the contract on chain via [`near-workspaces`](https://crates.io/crates/near-workspaces).
    - Sends transactions to the deployed contract to verify plugin functionality.

## Traits and their implementations

Traits doesn't contain any implementation, even though some interfaces are self-contained enough to have it.
It is this way since `near` macro from near-sdk-rs will only expose as public methods those that are implemented
during the trait implementation for the contract.

In the documentation all comments under Default Implementation makes remarks about the current implementation derived
automatically from macros. They can be changed if the trait is manually implemented rather than deriving the macro.

## Contributor Notes

[Contracts used in tests](/near-plugins-derive/tests/contracts/) set `channel = <MSRV>` in their `rust-toolchain` to make tests ensure that plugins are compatible with the MSRV. Developers working on this repo might want to locally set `channel = <MSRV>` in the root `./rust-toolchain` to surface incompabilities with the MSRV early on.

Why not commit `channel = <MSRV>` to `./rust-toolchain`? As a library crate we should leave the choice of the channel to users. Moreover, users should rather use a recent channel instead of the MSRV.

When compiling tests for the first time on a machine using the MSRV 1.69.0, an error might occur due to some dependencies of `near-workspaces` requiring a higher version of Rust. You can execute [./script/fix-dependencies.sh](./scripts/fix_dependencies.sh) to install a compatible version of these dependencies. The comments in that script provide additional information.


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

