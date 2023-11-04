#!/usr/bin/env bash

# Due to upgrades of dependencies of `near-workspaces`, the compilation of tests using the MSRV (min
# supported Rust version) may fail. This can be fixed be downgrading these dependencies to a version
# that supports our MRSV, which is what this script does.
#
# Reference: https://github.com/near/near-workspaces-rs/issues/336
#
# For some packages, `near-workspaces@0.9` depends on two different versions, requiring below
# downgrade commands to specify the full semver version as in `-p clap@4.4.7`. I assume once a new
# version of `clap` is released, say 4.4.8, then below must be updated to `-p clap@4.4.8`. Even
# though this is flaky, it seems to be cleanest approach that works in with CI (see #119 for some
# other attempts and how they failed in CI).
cargo update -p anstyle@1.0.4 --precise 1.0.2
cargo update -p anstyle-parse@0.2.2 --precise 0.2.1
cargo update -p clap@4.4.7 --precise 4.3.24
cargo update -p clap_lex@0.5.1 --precise 0.5.0
