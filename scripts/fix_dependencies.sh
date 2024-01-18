#!/usr/bin/env bash

# Due to upgrades of dependencies of `near-workspaces`, the compilation of tests with the MSRV (min
# supported Rust version) may fail. This can be fixed by downgrading these dependencies to a version
# that supports our MRSV, which is the purpose of this script.
#
# Reference: https://github.com/near/near-workspaces-rs/issues/336
#
# For some packages, `near-workspaces@0.9` depends on two different versions, requiring below
# downgrade commands to specify the full semver version as in `-p clap@4.4.7`. I assume once a new
# version of `clap` is released, say 4.4.8, then below must be changed to `-p clap@4.4.8`. Even
# though this requires maintenance, it seems to be cleanest approach that works with CI (see #119
# for some other attempts and how they failed in CI).
cargo update -p ahash@0.8.7 --precise 0.8.4
cargo update -p anstyle@1.0.4 --precise 1.0.2
cargo update -p anstyle-parse@0.2.3 --precise 0.2.1
cargo update -p anstyle-query@1.0.2 --precise 1.0.0
cargo update -p cargo-platform@0.1.6 --precise 0.1.5
cargo update -p clap@4.4.18 --precise 4.3.24
cargo update -p clap_lex@0.5.1 --precise 0.5.0
cargo update -p colored@2.1.0 --precise 2.0.4
cargo update -p home@0.5.9 --precise 0.5.5
