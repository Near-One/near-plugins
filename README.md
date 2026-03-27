# near-plugins

Derive macros that implement common patterns for NEAR smart contracts: access control, pausability, and upgradability.

```toml
[dependencies]
near-plugins = "0.6"
```

## Plugins

- [**AccessControllable**](#accesscontrollable) -- Role-based access control with super-admins, role admins, and grantees
- [**Ownable**](#ownable) -- Single-owner access control
- [**Pausable**](#pausable) -- Emergency stop mechanism with per-feature granularity (requires AccessControllable)
- [**Upgradable**](#upgradable) -- Two-phase code upgrades with optional staging delays (requires AccessControllable)

All state-changing operations emit [NEP-297](https://nomicon.io/Standards/EventsFormat) events.

---

### AccessControllable

Role-based access control inspired by [OpenZeppelin's AccessControl](https://docs.openzeppelin.com/contracts/5.x/api/access#AccessControl). Define roles as an enum, then use them to gate contract methods.

**Concepts:**
- **Super-admins** are admins for *every* role (but are not automatically grantees)
- **Admins** can grant and revoke a specific role
- **Grantees** hold a role and can call methods restricted to it

```rust
use near_plugins::{access_control, access_control_any, AccessControlRole};
use near_sdk::{near, PanicOnDefault};

#[derive(AccessControlRole, Deserialize, Serialize, Copy, Clone)]
#[serde(crate = "near_sdk::serde")]
pub enum Role {
    Minter,
    Burner,
}

#[access_control(role_type(Role))]
#[near(contract_state)]
#[derive(PanicOnDefault)]
pub struct Token {
    supply: u128,
}

#[near]
impl Token {
    #[init]
    pub fn new() -> Self {
        let mut contract = Self { supply: 0 };
        contract.acl_init_super_admin(near_sdk::env::current_account_id());
        contract
    }

    #[access_control_any(roles(Role::Minter))]
    pub fn mint(&mut self, amount: u128) {
        self.supply += amount;
    }

    #[access_control_any(roles(Role::Burner))]
    pub fn burn(&mut self, amount: u128) {
        self.supply -= amount;
    }
}
```

<details>
<summary>Key methods</summary>

| Method | Description |
|---|---|
| `acl_init_super_admin(account_id)` | Set the first super-admin (call once, typically in `#[init]`) |
| `acl_add_super_admin(account_id)` | Add a super-admin (caller must be super-admin) |
| `acl_revoke_super_admin(account_id)` | Revoke a super-admin |
| `acl_transfer_super_admin(account_id)` | Transfer super-admin from caller to another account |
| `acl_grant_role(role, account_id)` | Grant a role (caller must be admin for that role) |
| `acl_revoke_role(role, account_id)` | Revoke a role |
| `acl_has_role(role, account_id)` | Check if account has a role |
| `acl_add_admin(role, account_id)` | Make account an admin for a role |
| `acl_is_admin(role, account_id)` | Check if account is admin for a role (includes super-admins) |
| `acl_get_super_admins(skip, limit)` | Paginated list of super-admins |
| `acl_get_grantees(role, skip, limit)` | Paginated list of grantees for a role |
| `acl_get_permissioned_accounts()` | Bulk retrieval of all permissions (may hit gas limits) |

See the full API in [`access_controllable.rs`](near-plugins/src/access_controllable.rs).

</details>

---

### Ownable

Simple single-owner access control. The owner can be a regular account or a contract (e.g. a DAO).

```rust
use near_plugins::{Ownable, only};
use near_sdk::{near, AccountId, PanicOnDefault};

#[near(contract_state)]
#[derive(Ownable, PanicOnDefault)]
pub struct Counter {
    value: u64,
}

#[near]
impl Counter {
    #[init]
    pub fn new(owner: AccountId) -> Self {
        let mut contract = Self { value: 0 };
        contract.owner_set(Some(owner));
        contract
    }

    /// Only the owner can call this.
    #[only(owner)]
    pub fn increment(&mut self) {
        self.value += 1;
    }

    /// Only the contract itself can call this (e.g. via a callback).
    #[only(self)]
    pub fn reset(&mut self) {
        self.value = 0;
    }
}
```

**`#[only]` variants:** `#[only(owner)]`, `#[only(self)]`, `#[only(self, owner)]`

<details>
<summary>Key methods</summary>

| Method | Description |
|---|---|
| `owner_get()` | Returns the current owner (`Option<AccountId>`) |
| `owner_set(owner)` | Transfer or remove ownership (only callable by current owner) |
| `owner_is()` | Returns `true` if the caller is the owner |

See the full API in [`ownable.rs`](near-plugins/src/ownable.rs).

</details>

---

### Pausable

Emergency stop mechanism with per-feature granularity. Pausing the special key `"ALL"` pauses every pausable method. Requires [AccessControllable](#accesscontrollable) for authorization.

```rust
use near_plugins::{
    access_control, access_control_any, pause, if_paused,
    AccessControlRole, Pausable,
};
use near_sdk::{near, PanicOnDefault};

#[derive(AccessControlRole, Deserialize, Serialize, Copy, Clone)]
#[serde(crate = "near_sdk::serde")]
pub enum Role {
    PauseManager,
    UnpauseManager,
    EmergencyWithdrawer,
}

#[access_control(role_type(Role))]
#[near(contract_state)]
#[derive(Pausable, PanicOnDefault)]
#[pausable(
    pause_roles(Role::PauseManager),
    unpause_roles(Role::UnpauseManager),
)]
pub struct Contract {
    balance: u128,
}

#[near]
impl Contract {
    /// Blocked when "deposit" (the method name) is paused.
    #[pause]
    pub fn deposit(&mut self, amount: u128) {
        self.balance += amount;
    }

    /// Blocked when "transfers" is paused, except EmergencyWithdrawer can always call.
    #[pause(name = "transfers", except(roles(Role::EmergencyWithdrawer)))]
    pub fn withdraw(&mut self, amount: u128) {
        self.balance -= amount;
    }

    /// Only callable WHEN "deposit" IS paused (escape hatch pattern).
    #[if_paused(name = "deposit")]
    pub fn emergency_withdraw(&mut self) {
        self.balance = 0;
    }
}
```

<details>
<summary>Key methods</summary>

| Method | Description |
|---|---|
| `pa_pause_feature(key)` | Pause a feature (caller must have a `pause_role`) |
| `pa_unpause_feature(key)` | Unpause a feature (caller must have an `unpause_role`) |
| `pa_is_paused(key)` | Check if a feature is paused |
| `pa_all_paused()` | Returns all currently paused feature keys |

See the full API in [`pausable.rs`](near-plugins/src/pausable.rs).

</details>

---

### Upgradable

Two-phase contract upgrades: stage code, then deploy it. An optional staging duration enforces a minimum delay between staging and deployment, giving users time to review or opt out. Requires [AccessControllable](#accesscontrollable) for authorization.

```rust
use near_plugins::{access_control, AccessControlRole, Upgradable};
use near_sdk::{near, PanicOnDefault};

#[derive(AccessControlRole, Deserialize, Serialize, Copy, Clone)]
#[serde(crate = "near_sdk::serde")]
pub enum Role {
    DAO,
    CodeStager,
    CodeDeployer,
    DurationManager,
}

#[access_control(role_type(Role))]
#[near(contract_state)]
#[derive(Upgradable, PanicOnDefault)]
#[upgradable(access_control_roles(
    code_stagers(Role::CodeStager, Role::DAO),
    code_deployers(Role::CodeDeployer, Role::DAO),
    duration_initializers(Role::DurationManager),
    duration_update_stagers(Role::DurationManager),
    duration_update_appliers(Role::DurationManager),
))]
pub struct Contract;
```

**Upgrade flow:**
1. Call `up_stage_code` with the new WASM binary attached
2. Wait for the staging duration (if set)
3. Call `up_deploy_code` with a hash of the staged code for verification
4. Optionally pass `function_call_args` to run a migration function after deployment (automatically rolls back on failure)

<details>
<summary>Key methods</summary>

| Method | Description |
|---|---|
| `up_stage_code()` | Stage new contract code (attached as call argument) |
| `up_deploy_code(hash, function_call_args)` | Deploy staged code; `hash` verifies integrity, `function_call_args` runs a post-deploy migration |
| `up_staged_code()` | Returns staged code bytes |
| `up_staged_code_hash()` | Returns base58-encoded SHA-256 hash of staged code |
| `up_get_delay_status()` | Returns current staging duration and timestamps |
| `up_init_staging_duration(duration)` | Set the initial staging duration |
| `up_stage_update_staging_duration(duration)` | Stage a change to the staging duration (subject to current delay) |
| `up_apply_update_staging_duration()` | Apply the staged duration update |

See the full API in [`upgradable.rs`](near-plugins/src/upgradable.rs).

</details>

> **Note:** After a successful deployment, staged code remains in storage (for rollback recovery). Call `up_stage_code` with empty input to clean it up and reclaim storage.

---

## Compatibility

| | Version |
|---|---|
| `near-sdk` | `5.25+` |
| Rust edition | 2024 |
| MSRV | 1.86.0 |

## More examples

- [Example contracts](near-plugins-derive/tests/contracts/) used in integration tests
- [Integration tests](near-plugins-derive/tests/) demonstrating each plugin end-to-end

## Contributing

The trait definitions in [`near-plugins/src/`](near-plugins/src/) define each plugin's public API. The proc-macro implementations live in [`near-plugins-derive/src/`](near-plugins-derive/src/). Use [`cargo-expand`](https://github.com/dtolnay/cargo-expand) to inspect generated code for a specific contract.

**Testing:** Integration tests compile contracts from `near-plugins-derive/tests/contracts/`, deploy them on-chain via [`near-workspaces`](https://crates.io/crates/near-workspaces), and send transactions to verify behavior. Test contracts pin `channel = <MSRV>` in their `rust-toolchain` to ensure plugin compatibility with the minimum supported version.

## License

[CC0-1.0](LICENSE)
