use near_plugins::{FullAccessKeyFallback, Ownable};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{near_bindgen, AccountId, PanicOnDefault};

/// Deriving `FullAccessKeyFallback` requires the contract to be `Ownable.`
#[near_bindgen]
#[derive(Ownable, FullAccessKeyFallback, PanicOnDefault, BorshDeserialize, BorshSerialize)]
pub struct Counter;

#[near_bindgen]
impl Counter {
    /// Optionally set the owner in the constructor.
    #[init]
    pub fn new(owner: Option<AccountId>) -> Self {
        let mut contract = Self;
        if owner.is_some() {
            contract.owner_set(owner);
        }
        contract
    }
}
