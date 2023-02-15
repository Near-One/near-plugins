//! A simple contract to be deployed via `Upgradable`.

use near_plugins::{Ownable, Upgradable};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{near_bindgen, PanicOnDefault};

/// The struct is the same as in the initial contract `../upgradable`, so no [state migration] is
/// required.
///
/// [state migration]: https://docs.near.org/develop/upgrade#migrating-the-state
#[near_bindgen]
#[derive(Ownable, Upgradable, PanicOnDefault, BorshDeserialize, BorshSerialize)]
pub struct Contract;

#[near_bindgen]
impl Contract {
    /// A method that is _not_ defined in the initial contract, so its existence proves the
    /// contract defined in this file was deployed.
    pub fn is_upgraded() -> bool {
        true
    }
}
