# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.5.1](https://github.com/near/near-plugins/releases/tag/near-plugins-v0.5.1) - 2026-04-08

### Added

- separate pause and unpause roles in Pausable plugin ([#156](https://github.com/near/near-plugins/pull/156))
- make the upgrade process safer ([#147](https://github.com/near/near-plugins/pull/147))
- *(upgradable)* [**breaking**] add `hash` parameter to `up_deploy_code` ([#117](https://github.com/near/near-plugins/pull/117))
- bump near-sdk to 5.1.0 ([#134](https://github.com/near/near-plugins/pull/134))
- *(Acl)* add `acl_add_super_admin` ([#98](https://github.com/near/near-plugins/pull/98))
- *(Acl)* add `acl_transfer_super_admin` ([#94](https://github.com/near/near-plugins/pull/94))
- *(Pausable)* [**breaking**] make `pa_[un]pause` return `bool` ([#91](https://github.com/near/near-plugins/pull/91))
- *(Acl)* add `acl_revoke_super_admin` ([#89](https://github.com/near/near-plugins/pull/89))
- *(FullAccessKeyFallback)* [**breaking**] remove the plugin ([#87](https://github.com/near/near-plugins/pull/87))
- *(Upgradable)* [**breaking**] enable batched fn call after deploy ([#86](https://github.com/near/near-plugins/pull/86))
- *(Upgradable)* [**breaking**] use Acl for authorization  ([#85](https://github.com/near/near-plugins/pull/85))
- *(AccessControllable)* add `acl_get_permissioned_accounts` ([#78](https://github.com/near/near-plugins/pull/78))
- add Acl introspection of role variants ([#75](https://github.com/near/near-plugins/pull/75))

### Fixed

- update `syn` crate to match `near-sdk` crate and to enable parsing of `#[unsafe(no_mangle)]` ([#171](https://github.com/near/near-plugins/pull/171))
- Change hash encoding, support passing in `code` without borsh ([#152](https://github.com/near/near-plugins/pull/152))
- ABI generation ([#146](https://github.com/near/near-plugins/pull/146))

### Other

- bump version to 0.6.0 and fix repository URLs ([#178](https://github.com/near/near-plugins/pull/178))
- bump near-sdk to 5.2 ([#137](https://github.com/near/near-plugins/pull/137))
- stop pinning a toolchain version ([#131](https://github.com/near/near-plugins/pull/131))
- add a change log ([#103](https://github.com/near/near-plugins/pull/103))
- *(Acl)* add module-level docs ([#99](https://github.com/near/near-plugins/pull/99))
- *(Upgradable)* remove confusing statement ([#97](https://github.com/near/near-plugins/pull/97))
- [**breaking**] avoid cloning storage prefixes ([#92](https://github.com/near/near-plugins/pull/92))
- *(Upgradable)* code removal after deplyoment ([#88](https://github.com/near/near-plugins/pull/88))
- move integration tests to `near-plugins-derive` ([#81](https://github.com/near/near-plugins/pull/81))
- deduplicate `Upgradable` docs and examples ([#80](https://github.com/near/near-plugins/pull/80))
- add integration tests for `Upgradable` ([#79](https://github.com/near/near-plugins/pull/79))
- Add support for the delayed upgrade ([#44](https://github.com/near/near-plugins/pull/44))
- add notes on upgrading vulnerable code ([#71](https://github.com/near/near-plugins/pull/71))
- extend `acl_revoke_admin` docs ([#68](https://github.com/near/near-plugins/pull/68))
- add backstage metadata
- add integration tests for `FullAccessKeyFallback` ([#66](https://github.com/near/near-plugins/pull/66))
- fix clippy errors in `near-plugins/tests` ([#63](https://github.com/near/near-plugins/pull/63))
- deduplicate `Ownable` docs and examples ([#62](https://github.com/near/near-plugins/pull/62))
- mention `Ownable::owner_is` fails in view calls ([#60](https://github.com/near/near-plugins/pull/60))
- add integration tests for `Ownable` plugin ([#59](https://github.com/near/near-plugins/pull/59))
- Acl test contract names of roles and methods ([#57](https://github.com/near/near-plugins/pull/57))
- deduplicate `AccessControllable` docs and examples ([#55](https://github.com/near/near-plugins/pull/55))
- describe architecture & testing; add doc comments ([#54](https://github.com/near/near-plugins/pull/54))
- deduplicate `Pausable` docs and examples ([#53](https://github.com/near/near-plugins/pull/53))
- add integration tests for `Pausable` plugins ([#51](https://github.com/near/near-plugins/pull/51))
- Pausable `except` works when `ALL` is paused ([#49](https://github.com/near/near-plugins/pull/49))
- Add details of Pausable integration with Acl ([#50](https://github.com/near/near-plugins/pull/50))
- Make Pausable use Acl for authorization ([#47](https://github.com/near/near-plugins/pull/47))
- Add examples on plugins usage ([#34](https://github.com/near/near-plugins/pull/34))
- Add fn acl_get_super_admins to trait AccessControllable ([#29](https://github.com/near/near-plugins/pull/29))
- Re-export crate to avoid dependency in contract ([#24](https://github.com/near/near-plugins/pull/24))
- Add fn acl_init_super_admin to trait ([#25](https://github.com/near/near-plugins/pull/25))
- Manage dependencies using cargo workspace ([#16](https://github.com/near/near-plugins/pull/16))
- Add custom compilation mechanism for tests
- Use latest rust edition
- Remove unchecked methods and expose expected methods
- Add `acl_get_{admins, grantees}()`
- Add tests for new functions to manage roles
- Add functions to manage roles
- Add `acl_init_super_admin`
- Update dev-dependency `workspaces`
- Remove acl_*_unchecked methods from trait
- verify previously ignored return values
- extend test_attribute_access_control_any
- verify super-admin privileges
- Implement and test super-admin functionality
- Implement and test admin functionality
- Merge pull request #5 from mooori/acl
- rename `want` -> `expected`
- add `test_attribute_access_control_any`
- Implement `#[access_control_any(roles(...))]`
- Verify role type satisfies trait bounds
- Remove method stub
- add tests for acl_grant_role_unchecked
- add Setup to avoid repetition
- add helpers via AccessControllableContract
- Add event RoleGranted
- Rename variants of Role enum
- Omit optional storage_prefix arg in test contract
- Remove `_bitflag` suffix from trait method names
- implement TryFrom<&str> instead of FromStr
- make trait method acl_storage_prefix static
- Add building blocks for Acl plugin
- Update Cargo.toml / README / LICENSE
- Nit on README
- Add factory upgrades to the roadmap
- Add tests behind not(wasm32) flag
- Update visibility for plugins objects
- Tests and Macro.
- Update macros and tests
- Event Ergonomic: Add TODO and description
- Add Event on attaching a full access key
- Add tests
- Update design and add tests
- Add support for events
- Add documentation
- Fix is_self
- Create NEAR Plugins
