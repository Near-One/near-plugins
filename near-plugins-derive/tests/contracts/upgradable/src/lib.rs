use near_plugins::{access_control, AccessControlRole, AccessControllable, Upgradable};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{near_bindgen, AccountId, Duration, PanicOnDefault};

// TODO add doc comments
#[derive(AccessControlRole, Deserialize, Serialize, Copy, Clone)]
#[serde(crate = "near_sdk::serde")]
pub enum Role {
    // May do anything
    DAO,
    CodeStager,
    CodeDeployer,
    DurationManager,
}

/// Deriving `Upgradable` requires the contract to be `AccessControllable`.
#[access_control(role_type(Role))]
#[near_bindgen]
#[derive(Upgradable, PanicOnDefault, BorshDeserialize, BorshSerialize)]
#[upgradable(access_control_roles(
    code_stagers(Role::CodeStager, Role::DAO),
    code_deployers(Role::CodeDeployer, Role::DAO),
    duration_initializers(Role::DurationManager, Role::DAO),
    duration_update_stagers(Role::DurationManager, Role::DAO),
    duration_update_appliers(Role::DurationManager, Role::DAO),
))]
pub struct Contract {
    dummy: u64, // TODO remove after `__acl` field was removed (#84)
}

#[near_bindgen]
impl Contract {
    // TODO update docs and comments
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
    pub fn new(dao: Option<AccountId>, staging_duration: Option<Duration>) -> Self {
        let mut contract = Self {
            dummy: 0,
            __acl: Default::default(), // TODO remove after merging #84
        };

        if let Some(account_id) = dao {
            // TODO add warning regarding `*_unchecked()`
            let res = contract
                .__acl
                .grant_role_unchecked(Role::DAO.into(), &account_id);
            assert_eq!(true, res, "Failed to grant role");
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
            // TODO try using up_init_staging_duration
            contract.up_set_staging_duration_unchecked(staging_duration);
        }

        contract
    }
}
