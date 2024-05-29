use near_plugins::{access_control, AccessControlRole, AccessControllable, Upgradable};
use near_sdk::borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::env;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{near, AccountId, Duration, PanicOnDefault};

/// Defines roles for access control of protected methods provided by the `Upgradable` plugin.
#[derive(AccessControlRole, Deserialize, Serialize, Copy, Clone)]
#[serde(crate = "near_sdk::serde")]
pub enum Role {
    /// May successfully call any of the protected `Upgradable` methods since below it is passed to
    /// every attribute of `access_control_roles`.
    ///
    /// Using this pattern grantees of a single role are authorized to call all `Upgradable`methods.
    DAO,
    /// May successfully call `Upgradable::up_stage_code`, but none of the other protected methods,
    /// since below is passed only to the `code_stagers` attribute.
    ///
    /// Using this pattern grantees of a role are authorized to call only one particular protected
    /// `Upgradable` method.
    CodeStager,
    /// May successfully call `Upgradable::up_deploy_code`, but none of the other protected methods,
    /// since below is passed only to the `code_deployers` attribute.
    ///
    /// Using this pattern grantees of a role are authorized to call only one particular protected
    /// `Upgradable` method.
    CodeDeployer,
    /// May successfully call `Upgradable` methods to initialize and update the staging duration
    /// since below it is passed to the attributes `duration_initializers`,
    /// `duration_update_stagers`, and `duration_update_appliers`.
    ///
    /// Using this pattern grantees of a single role are authorized to call multiple (but not all)
    /// protected `Upgradable` methods.
    DurationManager,
}

/// Deriving `Upgradable` requires the contract to be `AccessControllable`.
///
/// Variants of `Role` are passed to `upgradables`'s `access_control_roles` attribute to specify
/// which roles are authorized to successfully call protected `Upgradable` methods. A protected
/// method panics if it is called by an account which is not a grantee of at least one of the
/// whitelisted roles.
#[access_control(role_type(Role))]
#[near(contract_state)]
#[derive(Upgradable, PanicOnDefault)]
#[upgradable(access_control_roles(
    code_stagers(Role::CodeStager, Role::DAO),
    code_deployers(Role::CodeDeployer, Role::DAO),
    duration_initializers(Role::DurationManager, Role::DAO),
    duration_update_stagers(Role::DurationManager, Role::DAO),
    duration_update_appliers(Role::DurationManager, Role::DAO),
))]
pub struct Contract;

#[near]
impl Contract {
    /// Makes the contract itself `AccessControllable` super admin to allow it granting and revoking
    /// permissions. If parameter `dao` is `Some(account_id)`, then `account_id` is granted
    /// `Role::DAO`. After initialization permissions can be managed using the methods provided by
    /// `AccessControllable`.
    ///
    /// Parameter `staging_duration` allows initializing the time that is required to pass between
    /// staging and deploying code. This delay provides a safety mechanism to protect users against
    /// unfavorable or malicious code upgrades. If `staging_duration` is `None`, no staging duration
    /// will be set in the constructor. It is possible to set it later using
    /// `Upgradable::up_init_staging_duration`. If no staging duration is set, it defaults to zero,
    /// allowing immediate deployments of staged code.
    #[init]
    pub fn new(dao: Option<AccountId>, staging_duration: Option<Duration>) -> Self {
        let mut contract = Self;

        // Make the contract itself access control super admin, allowing it to grant and revoke
        // permissions.
        near_sdk::require!(
            contract.acl_init_super_admin(env::current_account_id()),
            "Failed to initialize super admin",
        );

        // Optionally grant `Role::DAO`.
        if let Some(account_id) = dao {
            let res = contract.acl_grant_role(Role::DAO.into(), account_id);
            assert_eq!(Some(true), res, "Failed to grant role");
        }

        // Optionally initialize the staging duration.
        if let Some(staging_duration) = staging_duration {
            // Temporarily grant `Role::DurationManager` to the contract to authorize it for
            // initializing the staging duration. Granting and revoking the role is possible since
            // the contract was made super admin above.
            contract.acl_grant_role(Role::DurationManager.into(), env::current_account_id());
            contract.up_init_staging_duration(staging_duration);
            contract.acl_revoke_role(Role::DurationManager.into(), env::current_account_id());
        }

        contract
    }

    /// Function to verify the contract was deployed and initialized successfully.
    pub fn is_set_up(&self) -> bool {
        true
    }
}
