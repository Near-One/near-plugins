use near_plugins::{Ownable, Upgradable};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{near_bindgen, AccountId, PanicOnDefault};

/// Deriving `Upgradable` requires the contract to be `Ownable.`
#[near_bindgen]
#[derive(Ownable, Upgradable, PanicOnDefault, BorshDeserialize, BorshSerialize)]
pub struct Contract;

#[near_bindgen]
impl Contract {
    /// Parameter `owner` allows setting the owner in the constructor if an `AccountId` is provided.
    /// If `owner` is `None`, no owner will be set in the constructor. After contract initialization
    /// it is possible to set an owner with `Ownable::owner_set`.
    ///
    /// Parameter `staging_duration` allows initializing the time that is required to pass between
    /// staging and deploying code. This delay provides a safety mechanism to protect users against
    /// unfavorable or malicious code upgrades. If `staging_duration` is `None`, no staging duration
    /// will be set in the constructor. It is possible to set it later using
    /// `Upgradable::up_init_staging_duration`. If no staging duration is set, it defaults to zero,
    /// allowing immediate deployments of staged code.
    ///
    /// Since this constructor uses an `*_unchecked` method, it should be combined with code
    /// deployment in a batch transaction.
    #[init]
    pub fn new(owner: Option<AccountId>) -> Self {
        let mut contract = Self;

        // Optionally set the owner.
        if owner.is_some() {
            contract.owner_set(owner);
        }

        contract
    }
}
