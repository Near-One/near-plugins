# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.5.1](https://github.com/near/near-plugins/releases/tag/near-plugins-derive-v0.5.1) - 2026-04-08

### Added

- Improve Pausable migration guide and add migration tests ([#163](https://github.com/near/near-plugins/pull/163))

### Fixed

- Update `syn` crate to match `near-sdk` crate and to enable parsing of `#[unsafe(no_mangle)]` ([#171](https://github.com/near/near-plugins/pull/171))

### Other

- Downgrade `near-sdk` dep to allow plugins for older versions of SDK ([#183](https://github.com/near/near-plugins/pull/183))
