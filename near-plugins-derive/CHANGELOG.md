# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.6.0](https://github.com/Near-One/near-plugins/compare/near-plugins-derive-v0.1.0...near-plugins-derive-v0.6.0) - 2026-04-03

### Added

- Improve Pausable Migration Guide and Add Migration Tests ([#163](https://github.com/Near-One/near-plugins/pull/163))
- separate pause and unpause roles in Pausable plugin ([#156](https://github.com/Near-One/near-plugins/pull/156))
- make the upgrade process safer ([#147](https://github.com/Near-One/near-plugins/pull/147))
- *(upgradable)* [**breaking**] add `hash` parameter to `up_deploy_code` ([#117](https://github.com/Near-One/near-plugins/pull/117))
- bump near-sdk to 5.1.0 ([#134](https://github.com/Near-One/near-plugins/pull/134))
- *(Acl)* add `acl_add_super_admin` ([#98](https://github.com/Near-One/near-plugins/pull/98))
- *(Acl)* add `acl_transfer_super_admin` ([#94](https://github.com/Near-One/near-plugins/pull/94))
- *(Pausable)* [**breaking**] make `pa_[un]pause` return `bool` ([#91](https://github.com/Near-One/near-plugins/pull/91))
- *(Acl)* add `acl_revoke_super_admin` ([#89](https://github.com/Near-One/near-plugins/pull/89))
- *(FullAccessKeyFallback)* [**breaking**] remove the plugin ([#87](https://github.com/Near-One/near-plugins/pull/87))
- *(Upgradable)* [**breaking**] enable batched fn call after deploy ([#86](https://github.com/Near-One/near-plugins/pull/86))
- *(Upgradable)* [**breaking**] use Acl for authorization  ([#85](https://github.com/Near-One/near-plugins/pull/85))
- *(AccessControllable)* [**breaking**] Remove `__acl` field ([#84](https://github.com/Near-One/near-plugins/pull/84))
- *(AccessControllable)* add `acl_get_permissioned_accounts` ([#78](https://github.com/Near-One/near-plugins/pull/78))
- add Acl introspection of role variants ([#75](https://github.com/Near-One/near-plugins/pull/75))

### Fixed

- update `syn` crate to match `near-sdk` crate and to enable parsing of `#[unsafe(no_mangle)]` ([#171](https://github.com/Near-One/near-plugins/pull/171))
- Change hash encoding, support passing in `code` without borsh ([#152](https://github.com/Near-One/near-plugins/pull/152))
- remove the limit on minor version from `near-sdk` dependency ([#145](https://github.com/Near-One/near-plugins/pull/145))
- make view methods pausable ([#144](https://github.com/Near-One/near-plugins/pull/144))
- doc comment on Role variant crashes macro ([#33](https://github.com/Near-One/near-plugins/pull/33))

### Other

- bump version to 0.6.0 and fix repository URLs ([#178](https://github.com/Near-One/near-plugins/pull/178))
- update `rust-toolchain` to 1.86 in pausable-old ([#175](https://github.com/Near-One/near-plugins/pull/175))
- verify the state after wrong migration ([#159](https://github.com/Near-One/near-plugins/pull/159))
- deploy test contracts with cargo-near ([#157](https://github.com/Near-One/near-plugins/pull/157))
- Test upgrade access_controlable ([#141](https://github.com/Near-One/near-plugins/pull/141))
- bump near-sdk to 5.2 ([#137](https://github.com/Near-One/near-plugins/pull/137))
- stop pinning a toolchain version ([#131](https://github.com/Near-One/near-plugins/pull/131))
- only use `borsh` re-exported by `near-sdk` ([#122](https://github.com/Near-One/near-plugins/pull/122))
- [**breaking**] update Rust version and `near-*` dependencies ([#118](https://github.com/Near-One/near-plugins/pull/118))
- clean up some code ([#102](https://github.com/Near-One/near-plugins/pull/102))
- *(Acl)* reorder state modifications ([#101](https://github.com/Near-One/near-plugins/pull/101))
- *(Ownable)* removing contract keys freezes owner ([#100](https://github.com/Near-One/near-plugins/pull/100))
- use `emit` to log events ([#96](https://github.com/Near-One/near-plugins/pull/96))
- *(Pausable)* improve if_paused error message ([#95](https://github.com/Near-One/near-plugins/pull/95))
- [**breaking**] avoid cloning storage prefixes ([#92](https://github.com/Near-One/near-plugins/pull/92))
- *(Acl)* extend coverage ([#90](https://github.com/Near-One/near-plugins/pull/90))
- *(Upgradable)* code removal after deplyoment ([#88](https://github.com/Near-One/near-plugins/pull/88))
- *(Upgradable)* transact with contract after deployment ([#83](https://github.com/Near-One/near-plugins/pull/83))
- move integration tests to `near-plugins-derive` ([#81](https://github.com/Near-One/near-plugins/pull/81))
- Add support for the delayed upgrade ([#44](https://github.com/Near-One/near-plugins/pull/44))
- justify a usage of `std::panic!` ([#72](https://github.com/Near-One/near-plugins/pull/72))
- make Acl role limit explicit ([#70](https://github.com/Near-One/near-plugins/pull/70))
- add backstage metadata
- describe architecture & testing; add doc comments ([#54](https://github.com/Near-One/near-plugins/pull/54))
- Remove workaround for fixed near-sdk issue ([#46](https://github.com/Near-One/near-plugins/pull/46))
- Make Pausable use Acl for authorization ([#47](https://github.com/Near-One/near-plugins/pull/47))
- Migrate from `near_sdk::collections` to `near_sdk::store` ([#37](https://github.com/Near-One/near-plugins/pull/37))
- Add fn acl_get_super_admins to trait AccessControllable ([#29](https://github.com/Near-One/near-plugins/pull/29))
- Use double underscore in generated variables ([#26](https://github.com/Near-One/near-plugins/pull/26))
- Remove system panic. Use near sdk built in panic ([#27](https://github.com/Near-One/near-plugins/pull/27))
- Make calls to trait methods fully qualified in acl plugin ([#21](https://github.com/Near-One/near-plugins/pull/21))
- Update syntax to pass `role_type` to acl ([#19](https://github.com/Near-One/near-plugins/pull/19))
- Re-export crate to avoid dependency in contract ([#24](https://github.com/Near-One/near-plugins/pull/24))
- Add fn acl_init_super_admin to trait ([#25](https://github.com/Near-One/near-plugins/pull/25))
- Removing code duplication ([#22](https://github.com/Near-One/near-plugins/pull/22))
- Manage dependencies using cargo workspace ([#16](https://github.com/Near-One/near-plugins/pull/16))
- Use latest rust edition
- Address comments from clippy
- Remove unchecked methods and expose expected methods
- Add functions to manage roles
- Add `acl_get_{admins, grantees}()`
- Add functions to manage roles
- Add `acl_init_super_admin`
- Remove acl_*_unchecked methods from trait
- Implement and test super-admin functionality
- Implement and test admin functionality
- avoid a Vec allocation
- Document heuristic to detect *Ext impl
- Validate input `permission` in `add_bearer()`
- pr review
- pr review
- Implement `#[access_control_any(roles(...))]`
- Extend `is_near_bindgen_wrapped_or_marshall`
- Verify role type satisfies trait bounds
- Remove method stub
- Add event RoleGranted
- Make some methods in trait impl #[private]
- Add `fn new_bitflags_type_ident`
- cleanup
- cleanup
- document mapping between roles and bitflags
- Generate bitflags dynamically
- Remove `_bitflag` suffix from trait method names
- implement TryFrom<&str> instead of FromStr
- make trait method acl_storage_prefix static
- Add building blocks for Acl plugin
