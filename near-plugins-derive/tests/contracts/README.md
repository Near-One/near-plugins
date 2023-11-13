Contains contracts that use the plugins provided by `near-plugins`.

These contracts are compiled during tests via `near-workspaces` and may serve as examples for smart contract developers.

# TODO: contract to test optional ACL arguments
- `#[access_control]` has optional arguments, e.g. `storage_prefix`.
- Add a contract which sets all those optional arguments.
- Purpose: docs/example + verify processing of the arguments
