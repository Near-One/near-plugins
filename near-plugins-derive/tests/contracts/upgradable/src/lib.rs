use near_plugins::{Ownable, Upgradable};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{near_bindgen, AccountId, Duration, PanicOnDefault};

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
    pub fn new(owner: Option<AccountId>, staging_duration: Option<Duration>) -> Self {
        let mut contract = Self;

        // Optionally set the owner.
        if owner.is_some() {
            contract.owner_set(owner);
        }

        // Optionally initialize the staging duration.
        if let Some(staging_duration) = staging_duration {
            // The owner (set above) might be an account other than the contract itself. In that
            // case `Upgradable::up_init_staging_duration` would fail, since only the Owner may call
            // it successfully. Therefore we are using an (internal) unchecked method here.
            //
            // Avoid using `*_unchecked` functions in public contract methods that are not protected
            // by access control. Otherwise there is a risk of unwanted state changes carried out by
            // malicious users. For this example, we assume the constructor is called in a batch
            // transaction together with code deployment.
            contract.up_set_staging_duration_unchecked(staging_duration);
        }

        contract
    }
}
