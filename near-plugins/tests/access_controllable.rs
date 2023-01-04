// Using `pub` to avoid invalid `dead_code` warnings, see
// https://users.rust-lang.org/t/invalid-dead-code-warning-for-submodule-in-integration-test/80259
pub mod common;

use common::access_controllable_contract::AccessControllableContract;
use common::utils::{
    assert_insufficient_acl_permissions, assert_private_method_failure, assert_success_with,
};
use near_sdk::serde_json::json;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::path::Path;
use workspaces::network::Sandbox;
use workspaces::result::ExecutionFinalResult;
use workspaces::{Account, AccountId, Contract, Worker};

const PROJECT_PATH: &str = "./tests/contracts/access_controllable";

/// All roles which are defined in the contract in [`PROJECT_PATH`].
const ALL_ROLES: [&str; 3] = ["Increaser", "Skipper", "Resetter"];

/// Bundles resources required in tests.
struct Setup {
    /// The worker interacting with the current sandbox.
    worker: Worker<Sandbox>,
    /// Deployed instance of the contract in [`PROJECT_PATH`].
    contract: AccessControllableContract,
    /// A newly created account (which differs from the contract).
    account: Account,
}

impl Setup {
    fn contract_account(&self) -> &Account {
        self.contract.contract().as_account()
    }

    /// Deploys the contract and calls the initialization method without passing any accounts to be
    /// added as admin or grantees.
    async fn new() -> anyhow::Result<Self> {
        Self::new_with_admins_and_grantees(Default::default(), Default::default()).await
    }

    /// Deploys the contract and passes `admins` and `grantees` to the initialization method. Note
    /// that accounts corresponding to the ids in `admins` and `grantees` are _not_ created.
    async fn new_with_admins_and_grantees(
        admins: HashMap<String, AccountId>,
        grantees: HashMap<String, AccountId>,
    ) -> anyhow::Result<Self> {
        let worker = workspaces::sandbox().await?;
        let wasm =
            common::repo::compile_project(&Path::new(PROJECT_PATH), "access_controllable").await?;
        let contract = AccessControllableContract::new(worker.dev_deploy(&wasm).await?);
        let account = worker.dev_create_account().await?;

        contract
            .contract()
            .call("new")
            .args_json(json!({
                "admins": admins,
                "grantees": grantees,
            }))
            .max_gas()
            .transact()
            .await?
            .into_result()?;

        Ok(Self {
            worker,
            contract,
            account,
        })
    }

    /// Returns a new account that is super-admin.
    async fn new_super_admin_account(&self) -> anyhow::Result<Account> {
        let account = self.worker.dev_create_account().await?;
        self.contract
            .acl_add_super_admin_unchecked(self.contract_account(), account.id())
            .await?
            .into_result()?;
        Ok(account)
    }

    /// Returns a new account that is admin for `roles`.
    async fn new_account_as_admin(&self, roles: &[&str]) -> anyhow::Result<Account> {
        let account = self.worker.dev_create_account().await?;
        for &role in roles {
            self.contract
                .acl_add_admin_unchecked(self.contract_account(), role, account.id())
                .await?
                .into_result()?;
        }
        Ok(account)
    }

    async fn new_account_with_roles(&self, roles: &[&str]) -> anyhow::Result<Account> {
        let account = self.worker.dev_create_account().await?;
        for &role in roles {
            self.contract
                .acl_grant_role_unchecked(self.contract_account(), role, account.id())
                .await?
                .into_result()?;
        }
        Ok(account)
    }
}

async fn call_skip_one(
    contract: &Contract,
    caller: &Account,
) -> workspaces::Result<ExecutionFinalResult> {
    caller
        .call(contract.id(), "skip_one")
        .args_json(())
        .max_gas()
        .transact()
        .await
}

/// Smoke test of contract setup and basic functionality.
#[tokio::test]
async fn test_increase_and_get_counter() -> anyhow::Result<()> {
    let Setup {
        contract, account, ..
    } = Setup::new().await?;
    let contract = contract.contract();

    account
        .call(contract.id(), "increase")
        .max_gas()
        .transact()
        .await?
        .into_result()?;

    let res: u64 = account
        .call(contract.id(), "get_counter")
        .view()
        .await?
        .json()?;

    assert_eq!(res, 1);
    Ok(())
}

#[tokio::test]
async fn test_acl_initialization_in_constructor() -> anyhow::Result<()> {
    let admin_id: AccountId = "admin.acl_test.near".parse().unwrap();
    let grantee_id: AccountId = "grantee.acl_test.near".parse().unwrap();
    let setup = Setup::new_with_admins_and_grantees(
        HashMap::from([("Increaser".to_string(), admin_id.clone())]),
        HashMap::from([("Resetter".to_string(), grantee_id.clone())]),
    )
    .await?;

    setup
        .contract
        .assert_acl_is_admin(true, "Increaser", &admin_id)
        .await;
    setup
        .contract
        .assert_acl_has_role(true, "Resetter", &grantee_id)
        .await;

    Ok(())
}

#[tokio::test]
async fn test_acl_is_super_admin() -> anyhow::Result<()> {
    let Setup {
        contract, account, ..
    } = Setup::new().await?;

    let is_super_admin = contract.acl_is_super_admin(&account, account.id()).await?;
    assert_eq!(is_super_admin, false);

    contract
        .acl_add_super_admin_unchecked(contract.contract().as_account(), account.id())
        .await?
        .into_result()?;

    let is_super_admin = contract.acl_is_super_admin(&account, account.id()).await?;
    assert_eq!(is_super_admin, true);

    Ok(())
}

#[tokio::test]
async fn test_acl_init_super_admin() -> anyhow::Result<()> {
    let Setup {
        worker,
        contract,
        account,
        ..
    } = Setup::new().await?;
    let contract_account = contract.contract().as_account();

    // Calling `acl_init_super_admin` after initialization adds super-admin.
    contract
        .assert_acl_is_super_admin(false, contract_account, account.id())
        .await;
    let res = contract
        .acl_init_super_admin(contract_account, account.id())
        .await?;
    assert_success_with(res, true);
    contract
        .assert_acl_is_super_admin(true, contract_account, account.id())
        .await;

    // Once there's a super-admin, `acl_init_super_admin` returns `false`.
    let res = contract
        .acl_init_super_admin(contract_account, account.id())
        .await?;
    assert_success_with(res, false);

    let other_account = worker.dev_create_account().await?;
    let res = contract
        .acl_init_super_admin(contract_account, other_account.id())
        .await?;
    assert_success_with(res, false);
    contract
        .assert_acl_is_super_admin(false, contract_account, other_account.id())
        .await;

    // When all super-admins have been removed, it succeeds again.
    let res = contract
        .acl_revoke_super_admin_unchecked(contract_account, account.id())
        .await?;
    assert_success_with(res, true);
    let res = contract
        .acl_init_super_admin(contract_account, other_account.id())
        .await?;
    assert_success_with(res, true);
    contract
        .assert_acl_is_super_admin(true, contract_account, other_account.id())
        .await;

    Ok(())
}

#[tokio::test]
async fn test_acl_add_super_admin_unchecked() -> anyhow::Result<()> {
    let Setup {
        contract, account, ..
    } = Setup::new().await?;
    let contract_account = contract.contract().as_account();

    contract
        .assert_acl_is_super_admin(false, contract_account, account.id())
        .await;
    let res = contract
        .acl_add_super_admin_unchecked(contract_account, account.id())
        .await?;
    assert_success_with(res, true);
    contract
        .assert_acl_is_super_admin(true, contract_account, account.id())
        .await;

    // Adding as super-admin again behaves as expected.
    let res = contract
        .acl_add_super_admin_unchecked(contract_account, account.id())
        .await?;
    assert_success_with(res, false);

    Ok(())
}

#[tokio::test]
async fn test_acl_revoke_super_admin_unchecked() -> anyhow::Result<()> {
    let setup = Setup::new().await?;
    let account = setup.new_super_admin_account().await?;

    setup
        .contract
        .assert_acl_is_super_admin(true, setup.contract_account(), account.id())
        .await;

    // Revoke an existing super-admin permission.
    let res = setup
        .contract
        .acl_revoke_super_admin_unchecked(setup.contract_account(), account.id())
        .await?;
    assert_success_with(res, true);
    setup
        .contract
        .assert_acl_is_super_admin(false, setup.contract_account(), account.id())
        .await;

    // Revoke from an account which is not super-admin.
    let res = setup
        .contract
        .acl_revoke_super_admin_unchecked(setup.contract_account(), account.id())
        .await?;
    assert_success_with(res, false);

    Ok(())
}

/// Verify that a super-admin is admin for every role.
#[tokio::test]
async fn test_super_admin_is_any_admin() -> anyhow::Result<()> {
    let setup = Setup::new().await?;
    let super_admin = setup.new_super_admin_account().await?;

    for role in ALL_ROLES {
        setup
            .contract
            .assert_acl_is_admin(true, role, super_admin.id())
            .await;
    }

    Ok(())
}

/// Verify that a super-admin may add admins for every role.
#[tokio::test]
async fn test_super_admin_may_add_any_admin() -> anyhow::Result<()> {
    let setup = Setup::new().await?;
    let super_admin = setup.new_super_admin_account().await?;

    for role in ALL_ROLES {
        let account = setup.worker.dev_create_account().await?;
        setup
            .contract
            .assert_acl_is_admin(false, role, account.id())
            .await;

        let res = setup
            .contract
            .acl_add_admin(&super_admin, role, account.id())
            .await?;
        assert_eq!(res, Some(true));
        setup
            .contract
            .assert_acl_is_admin(true, role, account.id())
            .await;
    }

    Ok(())
}

/// Verify that a super-admin may revoke admins for every role.
#[tokio::test]
async fn test_super_admin_may_revoke_any_admin() -> anyhow::Result<()> {
    let setup = Setup::new().await?;
    let super_admin = setup.new_super_admin_account().await?;

    for role in ALL_ROLES {
        let admin = setup.new_account_as_admin(&[role]).await?;
        setup
            .contract
            .assert_acl_is_admin(true, role, admin.id())
            .await;

        let res = setup
            .contract
            .acl_revoke_admin(&super_admin, role, admin.id())
            .await?;
        assert_eq!(res, Some(true));
        setup
            .contract
            .assert_acl_is_admin(false, role, admin.id())
            .await;
    }
    Ok(())
}

/// Verify that a super-admin may grant every role.
#[tokio::test]
async fn test_super_admin_may_grant_any_role() -> anyhow::Result<()> {
    // TODO once acl_grant_role is implemented
    Ok(())
}

/// Verify that a super-admin may revoke every role.
#[tokio::test]
async fn test_super_admin_may_revoke_any_role() -> anyhow::Result<()> {
    // TODO once acl_revoke_role is implemented
    Ok(())
}

#[tokio::test]
async fn test_acl_is_admin() -> anyhow::Result<()> {
    let Setup {
        contract, account, ..
    } = Setup::new().await?;
    let contract_account = contract.contract().as_account();
    let role = "Increaser";

    let is_admin = contract.acl_is_admin(&account, role, account.id()).await?;
    assert_eq!(is_admin, false);

    contract
        .acl_add_admin_unchecked(contract_account, role, account.id())
        .await?
        .into_result()?;

    let is_admin = contract.acl_is_admin(&account, role, account.id()).await?;
    assert_eq!(is_admin, true);

    Ok(())
}

#[tokio::test]
async fn test_acl_add_admin() -> anyhow::Result<()> {
    let Setup {
        worker,
        contract,
        account,
        ..
    } = Setup::new().await?;
    let contract_account = contract.contract().as_account();
    let role = "Increaser";

    let acc_adding_admin = account;
    let acc_to_be_admin = worker.dev_create_account().await?;

    contract
        .assert_acl_is_admin(false, role, acc_to_be_admin.id())
        .await;

    // An account which isn't admin can't add admins.
    let added = contract
        .acl_add_admin(&acc_adding_admin, role, acc_to_be_admin.id())
        .await?;
    assert_eq!(added, None);

    // Admin can add others as admin.
    contract
        .acl_add_admin_unchecked(contract_account, role, acc_adding_admin.id())
        .await?
        .into_result()?;
    let added = contract
        .acl_add_admin(&acc_adding_admin, role, acc_to_be_admin.id())
        .await?;
    assert_eq!(added, Some(true));
    contract
        .assert_acl_is_admin(true, role, acc_to_be_admin.id())
        .await;

    // Adding an account that is already admin.
    let added = contract
        .acl_add_admin(&acc_adding_admin, role, acc_to_be_admin.id())
        .await?;
    assert_eq!(added, Some(false));

    Ok(())
}

#[tokio::test]
async fn test_acl_add_admin_unchecked() -> anyhow::Result<()> {
    let Setup {
        contract, account, ..
    } = Setup::new().await?;
    let contract_account = contract.contract().as_account();
    let role = "Increaser";

    contract
        .assert_acl_is_admin(false, role, account.id())
        .await;
    let res = contract
        .acl_add_admin_unchecked(contract_account, role, account.id())
        .await?;
    assert_success_with(res, true);
    contract.assert_acl_is_admin(true, role, account.id()).await;

    // Adding as admin again behaves as expected.
    let res = contract
        .acl_add_admin_unchecked(contract_account, role, account.id())
        .await?;
    assert_success_with(res, false);

    Ok(())
}

#[tokio::test]
async fn test_acl_revoke_admin() -> anyhow::Result<()> {
    let setup = Setup::new().await?;
    let role = "Skipper";
    let admin = setup.new_account_as_admin(&[role]).await?;

    setup
        .contract
        .assert_acl_is_admin(true, role, admin.id())
        .await;

    // Revoke is a no-op if revoker is not an admin for the role.
    let revoker = setup.new_account_as_admin(&[]).await?;
    let res = setup
        .contract
        .acl_revoke_admin(&revoker, role, admin.id())
        .await?;
    assert_eq!(res, None);
    let revoker = setup.new_account_as_admin(&["Increaser"]).await?;
    let res = setup
        .contract
        .acl_revoke_admin(&revoker, role, admin.id())
        .await?;
    assert_eq!(res, None);
    setup
        .contract
        .assert_acl_is_admin(true, role, admin.id())
        .await;

    // Revoke succeeds if the revoker is an admin for the role.
    let revoker = setup.new_account_as_admin(&[role]).await?;
    let res = setup
        .contract
        .acl_revoke_admin(&revoker, role, admin.id())
        .await?;
    assert_eq!(res, Some(true));
    setup
        .contract
        .assert_acl_is_admin(false, role, admin.id())
        .await;

    // Revoking a role for which the account isn't admin returns `Some(false)`.
    let revoker = setup.new_account_as_admin(&[role]).await?;
    let account = setup.worker.dev_create_account().await?;
    let res = setup
        .contract
        .acl_revoke_admin(&revoker, role, account.id())
        .await?;
    assert_eq!(res, Some(false));

    Ok(())
}

#[tokio::test]
async fn test_acl_renounce_admin() -> anyhow::Result<()> {
    let setup = Setup::new().await?;
    let role = "Resetter";

    // An account which is isn't admin calls `acl_renounce_admin`.
    let res = setup
        .contract
        .acl_renounce_admin(&setup.account, role)
        .await?;
    assert_eq!(res, false);

    // An admin calls `acl_renounce_admin`.
    let admin = setup.new_account_as_admin(&[role]).await?;
    setup
        .contract
        .assert_acl_is_admin(true, role, admin.id())
        .await;
    let res = setup.contract.acl_renounce_admin(&admin, role).await?;
    assert_eq!(res, true);
    setup
        .contract
        .assert_acl_is_admin(false, role, admin.id())
        .await;

    Ok(())
}

#[tokio::test]
async fn test_acl_revoke_admin_unchecked() -> anyhow::Result<()> {
    let setup = Setup::new().await?;
    let account = setup
        .new_account_as_admin(&["Increaser", "Resetter"])
        .await?;

    setup
        .contract
        .assert_acl_is_admin(true, "Increaser", account.id())
        .await;
    setup
        .contract
        .assert_acl_is_admin(true, "Resetter", account.id())
        .await;

    // Revoke admin permissions for one of the roles.
    let res = setup
        .contract
        .acl_revoke_admin_unchecked(setup.contract_account(), "Increaser", account.id())
        .await?;
    assert_success_with(res, true);
    setup
        .contract
        .assert_acl_is_admin(false, "Increaser", account.id())
        .await;
    setup
        .contract
        .assert_acl_is_admin(true, "Resetter", account.id())
        .await;

    // Revoke admin permissions for the other role too.
    let res = setup
        .contract
        .acl_revoke_admin_unchecked(setup.contract_account(), "Resetter", account.id())
        .await?;
    assert_success_with(res, true);
    setup
        .contract
        .assert_acl_is_admin(false, "Increaser", account.id())
        .await;
    setup
        .contract
        .assert_acl_is_admin(false, "Resetter", account.id())
        .await;

    // Revoking behaves as expected if the permission is not present.
    let res = setup
        .contract
        .acl_revoke_admin_unchecked(setup.contract_account(), "Resetter", account.id())
        .await?;
    assert_success_with(res, false);

    Ok(())
}

#[tokio::test]
async fn test_acl_has_role() -> anyhow::Result<()> {
    let Setup {
        contract, account, ..
    } = Setup::new().await?;
    let contract_account = contract.contract().as_account();
    let role = "Increaser";

    let has_role = contract.acl_has_role(&account, role, account.id()).await?;
    assert_eq!(has_role, false);

    contract
        .acl_grant_role_unchecked(contract_account, role, account.id())
        .await?
        .into_result()?;

    let has_role = contract.acl_has_role(&account, role, account.id()).await?;
    assert_eq!(has_role, true);

    Ok(())
}

#[tokio::test]
async fn test_acl_grant_role() -> anyhow::Result<()> {
    let Setup {
        worker,
        contract,
        account,
        ..
    } = Setup::new().await?;
    let contract_account = contract.contract().as_account();
    let role = "Skipper";

    let granter = account;
    let grantee = worker.dev_create_account().await?;

    // An account which isn't admin can't grant the role.
    contract
        .assert_acl_is_admin(false, role, granter.id())
        .await;
    let granted = contract
        .acl_grant_role(&granter, role, grantee.id())
        .await?;
    assert_eq!(granted, None);
    contract
        .assert_acl_has_role(false, role, grantee.id())
        .await;

    // Admin can grant the role.
    contract
        .acl_add_admin_unchecked(contract_account, role, granter.id())
        .await?
        .into_result()?;
    let granted = contract
        .acl_grant_role(&granter, role, grantee.id())
        .await?;
    assert_eq!(granted, Some(true));
    contract.assert_acl_has_role(true, role, grantee.id()).await;

    // Granting the role to an account which already is a grantee.
    let granted = contract
        .acl_grant_role(&granter, role, grantee.id())
        .await?;
    assert_eq!(granted, Some(false));

    Ok(())
}

#[tokio::test]
async fn test_acl_grant_role_unchecked() -> anyhow::Result<()> {
    let Setup {
        contract, account, ..
    } = Setup::new().await?;
    let contract_account = contract.contract().as_account();
    let role = "Increaser";

    contract
        .assert_acl_has_role(false, role, account.id())
        .await;
    let res = contract
        .acl_grant_role_unchecked(contract_account, role, account.id())
        .await?;
    assert_success_with(res, true);
    contract.assert_acl_has_role(true, role, account.id()).await;

    // Granting a role again behaves as expected.
    let res = contract
        .acl_grant_role_unchecked(contract_account, role, account.id())
        .await?;
    assert_success_with(res, false);

    Ok(())
}

#[tokio::test]
async fn test_acl_revoke_role() -> anyhow::Result<()> {
    let setup = Setup::new().await?;
    let role = "Skipper";
    let grantee = setup.new_account_with_roles(&[role]).await?;

    setup
        .contract
        .assert_acl_has_role(true, role, grantee.id())
        .await;

    // Revoke is a no-op if revoker is not an admin for the role.
    let revoker = setup.new_account_as_admin(&[]).await?;
    let res = setup
        .contract
        .acl_revoke_role(&revoker, role, grantee.id())
        .await?;
    assert_eq!(res, None);
    let revoker = setup.new_account_as_admin(&["Increaser"]).await?;
    let res = setup
        .contract
        .acl_revoke_role(&revoker, role, grantee.id())
        .await?;
    assert_eq!(res, None);
    setup
        .contract
        .assert_acl_has_role(true, role, grantee.id())
        .await;

    // Revoke succeeds if the revoker is an admin for the role.
    let revoker = setup.new_account_as_admin(&[role]).await?;
    let res = setup
        .contract
        .acl_revoke_role(&revoker, role, grantee.id())
        .await?;
    assert_eq!(res, Some(true));
    setup
        .contract
        .assert_acl_has_role(false, role, grantee.id())
        .await;

    // Revoking a role that isn't granted returns `Some(false)`.
    let revoker = setup.new_account_as_admin(&[role]).await?;
    let account = setup.worker.dev_create_account().await?;
    let res = setup
        .contract
        .acl_revoke_role(&revoker, role, account.id())
        .await?;
    assert_eq!(res, Some(false));

    Ok(())
}

#[tokio::test]
async fn test_acl_renounce_role() -> anyhow::Result<()> {
    let setup = Setup::new().await?;
    let role = "Resetter";

    // An account which is isn't grantee calls `acl_renounce_role`.
    let res = setup
        .contract
        .acl_renounce_role(&setup.account, role)
        .await?;
    assert_eq!(res, false);

    // A grantee calls `acl_renounce_admin`.
    let grantee = setup.new_account_with_roles(&[role]).await?;
    setup
        .contract
        .assert_acl_has_role(true, role, grantee.id())
        .await;
    let res = setup.contract.acl_renounce_role(&grantee, role).await?;
    assert_eq!(res, true);
    setup
        .contract
        .assert_acl_has_role(false, role, grantee.id())
        .await;

    Ok(())
}

#[tokio::test]
async fn test_acl_revoke_role_unchecked() -> anyhow::Result<()> {
    let setup = Setup::new().await?;
    let account = setup
        .new_account_with_roles(&["Increaser", "Resetter"])
        .await?;

    setup
        .contract
        .assert_acl_has_role(true, "Increaser", account.id())
        .await;
    setup
        .contract
        .assert_acl_has_role(true, "Resetter", account.id())
        .await;

    // Revoke one of the roles.
    let res = setup
        .contract
        .acl_revoke_role_unchecked(setup.contract_account(), "Increaser", account.id())
        .await?;
    assert_success_with(res, true);
    setup
        .contract
        .assert_acl_has_role(false, "Increaser", account.id())
        .await;
    setup
        .contract
        .assert_acl_has_role(true, "Resetter", account.id())
        .await;

    // Revoke the other role too.
    let res = setup
        .contract
        .acl_revoke_role_unchecked(setup.contract_account(), "Resetter", account.id())
        .await?;
    assert_success_with(res, true);
    setup
        .contract
        .assert_acl_has_role(false, "Increaser", account.id())
        .await;
    setup
        .contract
        .assert_acl_has_role(false, "Resetter", account.id())
        .await;

    // Revoking behaves as expected if the role is not granted.
    let res = setup
        .contract
        .acl_revoke_role_unchecked(setup.contract_account(), "Resetter", account.id())
        .await?;
    assert_success_with(res, false);

    Ok(())
}

#[tokio::test]
async fn test_attribute_access_control_any() -> anyhow::Result<()> {
    let setup = Setup::new().await?;
    let raw_contract = setup.contract.contract();
    let method_name = "skip_one";
    let allowed_roles = vec!["Increaser".to_string(), "Skipper".to_string()];

    // Account without any of the required permissions is restricted.
    let account = setup.new_account_with_roles(&[]).await?;
    let res = call_skip_one(raw_contract, &account).await?;
    assert_insufficient_acl_permissions(res, method_name, allowed_roles.clone());
    let account = setup.new_account_with_roles(&["Resetter"]).await?;
    let res = call_skip_one(raw_contract, &account).await?;
    assert_insufficient_acl_permissions(res, method_name, allowed_roles.clone());

    // A super-admin which has not been granted the role is restricted.
    let super_admin = setup.new_super_admin_account().await?;
    let res = call_skip_one(raw_contract, &super_admin).await?;
    assert_insufficient_acl_permissions(res, method_name, allowed_roles.clone());

    // An admin for a permitted role is restricted (no grantee of role itself).
    let admin = setup.new_account_as_admin(&["Increaser"]).await?;
    let res = call_skip_one(raw_contract, &admin).await?;
    assert_insufficient_acl_permissions(res, method_name, allowed_roles.clone());

    // Account with one of the required permissions succeeds.
    let account = setup.new_account_with_roles(&["Increaser"]).await?;
    let res = call_skip_one(raw_contract, &account).await?;
    assert_success_with(res, 2);
    let account = setup.new_account_with_roles(&["Skipper"]).await?;
    let res = call_skip_one(raw_contract, &account).await?;
    assert_success_with(res, 4);
    let account = setup
        .new_account_with_roles(&["Increaser", "Resetter"])
        .await?;
    let res = call_skip_one(raw_contract, &account).await?;
    assert_success_with(res, 6);

    // Account with both permissions succeeds.
    let account = setup
        .new_account_with_roles(&["Increaser", "Skipper"])
        .await?;
    let res = call_skip_one(raw_contract, &account).await?;
    assert_success_with(res, 8);
    let account = setup
        .new_account_with_roles(&["Increaser", "Skipper", "Resetter"])
        .await?;
    let res = call_skip_one(raw_contract, &account).await?;
    assert_success_with(res, 10);

    Ok(())
}

#[tokio::test]
async fn test_acl_init_super_admin_is_private() -> anyhow::Result<()> {
    let Setup {
        contract, account, ..
    } = Setup::new().await?;
    let res = contract
        .acl_init_super_admin(&account, account.id())
        .await?;
    assert_private_method_failure(res, "acl_init_super_admin");
    Ok(())
}

#[tokio::test]
async fn test_acl_get_super_admins() -> anyhow::Result<()> {
    let setup = Setup::new().await?;

    let super_admin_ids = vec![
        setup.new_super_admin_account().await?,
        setup.new_super_admin_account().await?,
        setup.new_super_admin_account().await?,
    ]
    .iter()
    .map(|account| account.id().clone())
    .collect::<Vec<_>>();

    // Behaves as expected for limit = 0.
    let actual = setup
        .contract
        .acl_get_super_admins(&setup.account, 0, 0)
        .await?;
    assert_eq!(actual, vec![],);

    // Skip outside of the number of existing super-admins.
    let n = u64::try_from(super_admin_ids.len()).unwrap();
    let actual = setup
        .contract
        .acl_get_super_admins(&setup.account, n, 1)
        .await?;
    assert_eq!(actual, vec![],);

    // Retrieve super-admins with step size 1.
    for i in 0..3 {
        let actual = setup
            .contract
            .acl_get_super_admins(&setup.account, i, 1)
            .await?;
        let i = usize::try_from(i).unwrap();
        let expected = super_admin_ids[i..i + 1].to_vec();
        assert_eq!(actual, expected, "Mismatch at position {}", i,);
    }

    // Retrieve super-admins with step size 2.
    let actual = setup
        .contract
        .acl_get_super_admins(&setup.account, 0, 2)
        .await?;
    let expected = super_admin_ids[0..2].to_vec();
    assert_eq!(actual, expected);
    let actual = setup
        .contract
        .acl_get_super_admins(&setup.account, 2, 2)
        .await?;
    let expected = vec![super_admin_ids[2].clone()];
    assert_eq!(actual, expected);

    // Retrieve all super-admins at once.
    let actual = setup
        .contract
        .acl_get_super_admins(&setup.account, 0, 3)
        .await?;
    assert_eq!(actual, super_admin_ids);

    // Limit larger than the number of existing super-admins.
    let actual = setup
        .contract
        .acl_get_super_admins(&setup.account, 0, 4)
        .await?;
    assert_eq!(actual, super_admin_ids);

    Ok(())
}

#[tokio::test]
async fn test_acl_get_admins() -> anyhow::Result<()> {
    let setup = Setup::new().await?;
    let role = "Skipper";

    let admin_ids = vec![
        setup.new_account_as_admin(&[role]).await?,
        setup.new_account_as_admin(&[role]).await?,
        setup.new_account_as_admin(&[role]).await?,
    ]
    .iter()
    .map(|account| account.id().clone())
    .collect::<Vec<_>>();

    // Behaves as expected for limit = 0.
    let actual = setup
        .contract
        .acl_get_admins(&setup.account, role, 0, 0)
        .await?;
    assert_eq!(actual, vec![],);

    // Skip outside of the number of existing admins.
    let n = u64::try_from(admin_ids.len()).unwrap();
    let actual = setup
        .contract
        .acl_get_admins(&setup.account, role, n, 1)
        .await?;
    assert_eq!(actual, vec![],);

    // Retrieve admins with step size 1.
    for i in 0..3 {
        let actual = setup
            .contract
            .acl_get_admins(&setup.account, role, i, 1)
            .await?;
        let i = usize::try_from(i).unwrap();
        let expected = admin_ids[i..i + 1].to_vec();
        assert_eq!(actual, expected, "Mismatch at position {}", i,);
    }

    // Retrieve admins with step size 2.
    let actual = setup
        .contract
        .acl_get_admins(&setup.account, role, 0, 2)
        .await?;
    let expected = admin_ids[0..2].to_vec();
    assert_eq!(actual, expected);
    let actual = setup
        .contract
        .acl_get_admins(&setup.account, role, 2, 2)
        .await?;
    let expected = vec![admin_ids[2].clone()];
    assert_eq!(actual, expected);

    // Retrieve all admins at once.
    let actual = setup
        .contract
        .acl_get_admins(&setup.account, role, 0, 3)
        .await?;
    assert_eq!(actual, admin_ids);

    // Limit larger than the number of existing admins.
    let actual = setup
        .contract
        .acl_get_admins(&setup.account, role, 0, 4)
        .await?;
    assert_eq!(actual, admin_ids);

    Ok(())
}

#[tokio::test]
async fn test_acl_get_grantees() -> anyhow::Result<()> {
    let setup = Setup::new().await?;
    let role = "Increaser";

    let grantee_ids = vec![
        setup.new_account_with_roles(&[role]).await?,
        setup.new_account_with_roles(&[role]).await?,
        setup.new_account_with_roles(&[role]).await?,
    ]
    .iter()
    .map(|account| account.id().clone())
    .collect::<Vec<_>>();

    // Behaves as expected for limit = 0.
    let actual = setup
        .contract
        .acl_get_grantees(&setup.account, role, 0, 0)
        .await?;
    assert_eq!(actual, vec![],);

    // Skip outside of the number of existing grantees.
    let n = u64::try_from(grantee_ids.len()).unwrap();
    let actual = setup
        .contract
        .acl_get_grantees(&setup.account, role, n, 1)
        .await?;
    assert_eq!(actual, vec![],);

    // Retrieve grantees with step size 1.
    for i in 0..3 {
        let actual = setup
            .contract
            .acl_get_grantees(&setup.account, role, i, 1)
            .await?;
        let i = usize::try_from(i).unwrap();
        let expected = grantee_ids[i..i + 1].to_vec();
        assert_eq!(actual, expected, "Mismatch at position {}", i,);
    }

    // Retrieve grantees with step size 2.
    let actual = setup
        .contract
        .acl_get_grantees(&setup.account, role, 0, 2)
        .await?;
    let expected = grantee_ids[0..2].to_vec();
    assert_eq!(actual, expected);
    let actual = setup
        .contract
        .acl_get_grantees(&setup.account, role, 2, 2)
        .await?;
    let expected = vec![grantee_ids[2].clone()];
    assert_eq!(actual, expected);

    // Retrieve all grantees at once.
    let actual = setup
        .contract
        .acl_get_grantees(&setup.account, role, 0, 3)
        .await?;
    assert_eq!(actual, grantee_ids);

    // Limit larger than the number of existing grantees.
    let actual = setup
        .contract
        .acl_get_grantees(&setup.account, role, 0, 4)
        .await?;
    assert_eq!(actual, grantee_ids);

    Ok(())
}

#[tokio::test]
async fn test_acl_add_super_admin_unchecked_is_private() -> anyhow::Result<()> {
    let Setup {
        contract, account, ..
    } = Setup::new().await?;
    let res = contract
        .acl_add_super_admin_unchecked(&account, account.id())
        .await?;
    assert_private_method_failure(res, "acl_add_super_admin_unchecked");
    Ok(())
}

#[tokio::test]
async fn test_acl_revoke_super_admin_unchecked_is_private() -> anyhow::Result<()> {
    let Setup {
        contract, account, ..
    } = Setup::new().await?;
    let res = contract
        .acl_revoke_super_admin_unchecked(&account, account.id())
        .await?;
    assert_private_method_failure(res, "acl_revoke_super_admin_unchecked");
    Ok(())
}

#[tokio::test]
async fn test_acl_add_admin_unchecked_is_private() -> anyhow::Result<()> {
    let Setup {
        contract, account, ..
    } = Setup::new().await?;
    let res = contract
        .acl_add_admin_unchecked(&account, "Increaser", account.id())
        .await?;
    assert_private_method_failure(res, "acl_add_admin_unchecked");
    Ok(())
}

#[tokio::test]
async fn test_acl_revoke_admin_unchecked_is_private() -> anyhow::Result<()> {
    let Setup {
        contract, account, ..
    } = Setup::new().await?;
    let res = contract
        .acl_revoke_admin_unchecked(&account, "Increaser", account.id())
        .await?;
    assert_private_method_failure(res, "acl_revoke_admin_unchecked");
    Ok(())
}

#[tokio::test]
async fn test_acl_grant_role_unchecked_is_private() -> anyhow::Result<()> {
    let Setup {
        contract, account, ..
    } = Setup::new().await?;
    let res = contract
        .acl_grant_role_unchecked(&account, "Increaser", account.id())
        .await?;
    assert_private_method_failure(res, "acl_grant_role_unchecked");
    Ok(())
}

#[tokio::test]
async fn test_acl_revoke_role_unchecked_is_private() -> anyhow::Result<()> {
    let Setup {
        contract, account, ..
    } = Setup::new().await?;
    let res = contract
        .acl_revoke_role_unchecked(&account, "Increaser", account.id())
        .await?;
    assert_private_method_failure(res, "acl_revoke_role_unchecked");
    Ok(())
}
