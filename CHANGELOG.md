# 0.2.0 (TBD)

## BREAKING CHANGES:

- [#118](https://github.com/aurora-is-near/near-plugins/pull/118): Update of `rust-version` (MSRV) from 1.64.0 to 1.69.0. Contracts using `near-plugins` now require a Rust version of at least 1.69.0.
  - Developers who want to run the test suite of `near-plugins` and run into compilation errors can follow [this workaround](https://github.com/aurora-is-near/near-plugins/pull/118#issuecomment-1794576809).

## Testing

- [128](https://github.com/aurora-is-near/near-plugins/pull/128) applies temporary workarounds required to build tests.

## Dependencies

### Dev-dependencies

- [#118](https://github.com/aurora-is-near/near-plugins/pull/118) upgrades:
  - `near-sdk`
  - `near-workspaces`
- [#122](https://github.com/aurora-is-near/near-plugins/pull/122) removes `borsh` which was an unused dev-dependencies.

# 0.1.0 (2023-05-08)

This section lists changes affecting multiple plugins. Changes that affect only individual plugins are listed below.

## BREAKING CHANGES
- Functions returning storage prefixes return `&'static [u8]` instead of `Vec<u8>`. [[#92](https://github.com/aurora-is-near/near-plugins/pull/92)]

## Testing & documentation
- All plugins are now tested in integration tests using Near’s [workspaces](https://docs.rs/crate/workspaces/0.7.0), as described [here](https://github.com/aurora-is-near/near-plugins#testing).
- The contracts used in tests contain many comments and serve as usage examples.

## Dependencies
- Update `near-sdk` to 4.1.0. [[#40](https://github.com/aurora-is-near/near-plugins/pull/40)]
- Update `workspaces` to 0.7. [[#65](https://github.com/aurora-is-near/near-plugins/pull/65)]

## Plugins

### `AccessControllable`

#### BREAKING CHANGES
- Store plugin state under a separate storage key. Previously it was stored under a field injected into the struct that derives `AccessControllable`. [[#84](https://github.com/aurora-is-near/near-plugins/pull/84)]

#### Feature enhancements
- Make the limit for the the number of role variants explicit. [[#70](https://github.com/aurora-is-near/near-plugins/pull/70)]
- Enable viewing permissions that have been granted to accounts. [[#75](https://github.com/aurora-is-near/near-plugins/pull/75), [#78](https://github.com/aurora-is-near/near-plugins/pull/78)]
- Add the function `acl_revoke_super_admin`. [[#89](https://github.com/aurora-is-near/near-plugins/pull/89)]
- Add the function `acl_transfer_super_admin`. [[#94](https://github.com/aurora-is-near/near-plugins/pull/94)]
- Add the function `acl_add_super_admin`. [[#98](https://github.com/aurora-is-near/near-plugins/pull/98)]

### `FullAccessKeyFallback`

#### BREAKING CHANGES
- The plugin has been removed. [[#87](https://github.com/aurora-is-near/near-plugins/pull/87)]

### `Pausable`

#### BREAKING CHANGES
- Use `AccessControllable` instead of `Ownable` to manage authorization of (un)pausing features and to define exemptions via `except`. [[47](https://github.com/aurora-is-near/near-plugins/pull/47)]
- Make functions `pa_[un]pause` return `bool`. [[#91](https://github.com/aurora-is-near/near-plugins/pull/91)]
- Emit events `Pause` and `Unpause` only if state was modified successfully. [[#91](https://github.com/aurora-is-near/near-plugins/pull/91)]

#### Feature enhancements
- Improve an error message related to `if_paused`. [[95](https://github.com/aurora-is-near/near-plugins/pull/95)]

### `Upgradable`

#### BREAKING CHANGES
- Use `AccessControllable` instead of `Ownable` to manage permissions for functionality provided by `Upgradable`. [[85](https://github.com/aurora-is-near/near-plugins/pull/85)]
- Enable optionally batching a function call with code deployment, which changes the signature of `Upgradable::up_deploy_code`. [[86](https://github.com/aurora-is-near/near-plugins/pull/86)]

#### Feature enhancements
- Allow contracts to set a minimum duration that must pass between staging and deploying new code. The staging duration is a safety mechanism to protect users. [[#44](https://github.com/aurora-is-near/near-plugins/pull/44)]
