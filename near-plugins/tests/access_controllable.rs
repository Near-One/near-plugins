mod common;

use common::access_controllable_contract::{AccessControllableContract, Caller};
use common::utils::{
    assert_insufficient_acl_permissions, assert_private_method_failure, assert_success_with,
};
use near_sdk::serde_json::json;
use workspaces::network::Sandbox;
use workspaces::result::ExecutionFinalResult;
use workspaces::{Account, Contract, Worker};

const PROJECT_PATH: &str = "./tests/contracts/access_controllable";

/// All roles which are defined in the contract in [`PROJECT_PATH`].
const ALL_ROLES: [&str; 3] = ["LevelA", "LevelB", "LevelC"];

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
    async fn new() -> anyhow::Result<Self> {
        let worker = workspaces::sandbox().await?;
        let wasm = workspaces::compile_project(PROJECT_PATH).await?;
        let contract = AccessControllableContract::new(worker.dev_deploy(&wasm).await?);
        let account = worker.dev_create_account().await?;

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
            .acl_add_super_admin_unchecked(Caller::Contract, account.id())
            .await?
            .into_result()?;
        Ok(account)
    }

    /// Returns a new account that is admin for `roles`.
    async fn new_account_as_admin(&self, roles: &[&str]) -> anyhow::Result<Account> {
        let account = self.worker.dev_create_account().await?;
        for &role in roles {
            self.contract
                .acl_add_admin_unchecked(Caller::Contract, role, account.id())
                .await?
                .into_result()?;
        }
        Ok(account)
    }

    async fn new_account_with_roles(&self, roles: &[&str]) -> anyhow::Result<Account> {
        let account = self.worker.dev_create_account().await?;
        for &role in roles {
            self.contract
                .acl_grant_role_unchecked(Caller::Contract, role, account.id())
                .await?
                .into_result()?;
        }
        Ok(account)
    }
}

/// Represents the outcome of a transaction sent to the [`PROJECT_PATH`]
/// contract.
// TODO generic `T` instead of `String`
#[derive(Debug)]
enum TxOutcome {
    Success(String),
    AclFailure(AclFailure),
}

#[derive(Debug)]
struct AclFailure {
    method_name: String,
    /// The roles that are allowed (specified in the contract).
    allowed_roles: Vec<String>,
    /// The result of the transaction. Not allowing view calls here since
    /// `ViewResultDetails` is not sufficient to verify ACL failure.
    result: ExecutionFinalResult,
}

impl TxOutcome {
    fn assert_success(&self, expected: String) {
        let got = match self {
            TxOutcome::Success(got) => got.clone(),
            TxOutcome::AclFailure(failure) => panic!(
                "Expected transaction success but it failed with: {:?}",
                failure
            ),
        };
        assert_eq!(got, expected);
    }

    fn assert_acl_failure(&self) {
        let failure = match self {
            TxOutcome::Success(_) => panic!("Expected transaction failure"),
            TxOutcome::AclFailure(failure) => failure,
        };
        assert_insufficient_acl_permissions(
            failure.result.clone(),
            failure.method_name.as_str(),
            failure.allowed_roles.clone(),
        );
    }
}

async fn call_restricted_greeting(
    contract: &Contract,
    caller: &Account,
) -> anyhow::Result<TxOutcome> {
    let res = caller
        .call(contract.id(), "restricted_greeting")
        .args_json(())
        .max_gas()
        .transact()
        .await?;
    let tx_outcome = match res.is_success() {
        true => TxOutcome::Success(res.into_result().unwrap().json::<String>().unwrap()),
        false => TxOutcome::AclFailure(AclFailure {
            method_name: "restricted_greeting".to_string(),
            allowed_roles: vec!["LevelA".to_string(), "LevelC".to_string()],
            result: res,
        }),
    };
    Ok(tx_outcome)
}

/// Smoke test of contract setup and basic functionality.
#[tokio::test]
async fn test_set_and_get_status() -> anyhow::Result<()> {
    let Setup {
        contract, account, ..
    } = Setup::new().await?;
    let contract = contract.contract();
    let message = "hello world";

    account
        .call(contract.id(), "set_status")
        .args_json(json!({
            "message": message,
        }))
        .max_gas()
        .transact()
        .await?
        .into_result()?;

    let res: String = account
        .call(contract.id(), "get_status")
        .args_json(json!({
            "account_id": account.id(),
        }))
        .view()
        .await?
        .json()?;

    assert_eq!(res, message);
    Ok(())
}

#[tokio::test]
async fn test_acl_is_super_admin() -> anyhow::Result<()> {
    let Setup {
        contract, account, ..
    } = Setup::new().await?;

    let is_super_admin = contract
        .acl_is_super_admin(account.clone().into(), account.id())
        .await?;
    assert_eq!(is_super_admin, false);

    contract
        .acl_add_super_admin_unchecked(Caller::Contract, account.id())
        .await?
        .into_result()?;

    let is_super_admin = contract
        .acl_is_super_admin(account.clone().into(), account.id())
        .await?;
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

    // Calling `acl_init_super_admin` after initialization adds super-admin.
    contract
        .assert_acl_is_super_admin(false, account.id())
        .await;
    let res = contract
        .acl_init_super_admin(Caller::Contract, account.id())
        .await?;
    assert_success_with(res, true);
    contract.assert_acl_is_super_admin(true, account.id()).await;

    // Once there's a super-admin, `acl_init_super_admin` returns `false`.
    let res = contract
        .acl_init_super_admin(Caller::Contract, account.id())
        .await?;
    assert_success_with(res, false);

    let other_account = worker.dev_create_account().await?;
    let res = contract
        .acl_init_super_admin(Caller::Contract, other_account.id())
        .await?;
    assert_success_with(res, false);
    contract
        .assert_acl_is_super_admin(false, other_account.id())
        .await;

    // When all super-admins have been removed, it succeeds again.
    let res = contract
        .acl_revoke_super_admin_unchecked(Caller::Contract, account.id())
        .await?;
    assert_success_with(res, true);
    let res = contract
        .acl_init_super_admin(Caller::Contract, other_account.id())
        .await?;
    assert_success_with(res, true);
    contract
        .assert_acl_is_super_admin(true, other_account.id())
        .await;

    Ok(())
}

#[tokio::test]
async fn test_acl_add_super_admin_unchecked() -> anyhow::Result<()> {
    let Setup {
        contract, account, ..
    } = Setup::new().await?;

    contract
        .assert_acl_is_super_admin(false, account.id())
        .await;
    let res = contract
        .acl_add_super_admin_unchecked(Caller::Contract, account.id())
        .await?;
    assert_success_with(res, true);
    contract.assert_acl_is_super_admin(true, account.id()).await;

    // Adding as super-admin again behaves as expected.
    let res = contract
        .acl_add_super_admin_unchecked(Caller::Contract, account.id())
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
        .assert_acl_is_super_admin(true, account.id())
        .await;

    // Revoke an existing super-admin permission.
    let res = setup
        .contract
        .acl_revoke_super_admin_unchecked(Caller::Contract, account.id())
        .await?;
    assert_success_with(res, true);
    setup
        .contract
        .assert_acl_is_super_admin(false, account.id())
        .await;

    // Revoke from an account which is not super-admin.
    let res = setup
        .contract
        .acl_revoke_super_admin_unchecked(Caller::Contract, account.id())
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
            .acl_add_admin(super_admin.clone().into(), role, account.id())
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
            .acl_revoke_admin(super_admin.clone().into(), role, admin.id())
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
    let role = "LevelA";

    let is_admin = contract
        .acl_is_admin(account.clone().into(), role, account.id())
        .await?;
    assert_eq!(is_admin, false);

    contract
        .acl_add_admin_unchecked(Caller::Contract, role, account.id())
        .await?
        .into_result()?;

    let is_admin = contract
        .acl_is_admin(account.clone().into(), role, account.id())
        .await?;
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
    let role = "LevelA";

    let acc_adding_admin = account;
    let acc_to_be_admin = worker.dev_create_account().await?;

    contract
        .assert_acl_is_admin(false, role, acc_to_be_admin.id())
        .await;

    // An account which isn't admin can't add admins.
    let added = contract
        .acl_add_admin(acc_adding_admin.clone().into(), role, acc_to_be_admin.id())
        .await?;
    assert_eq!(added, None);

    // Admin can add others as admin.
    contract
        .acl_add_admin_unchecked(Caller::Contract, role, acc_adding_admin.id())
        .await?
        .into_result()?;
    let added = contract
        .acl_add_admin(acc_adding_admin.clone().into(), role, acc_to_be_admin.id())
        .await?;
    assert_eq!(added, Some(true));
    contract
        .assert_acl_is_admin(true, role, acc_to_be_admin.id())
        .await;

    // Adding an account that is already admin.
    let added = contract
        .acl_add_admin(acc_adding_admin.clone().into(), role, acc_to_be_admin.id())
        .await?;
    assert_eq!(added, Some(false));

    Ok(())
}

#[tokio::test]
async fn test_acl_add_admin_unchecked() -> anyhow::Result<()> {
    let Setup {
        contract, account, ..
    } = Setup::new().await?;
    let role = "LevelA";

    contract
        .assert_acl_is_admin(false, role, account.id())
        .await;
    let res = contract
        .acl_add_admin_unchecked(Caller::Contract, role, account.id())
        .await?;
    assert_success_with(res, true);
    contract.assert_acl_is_admin(true, role, account.id()).await;

    // Adding as admin again behaves as expected.
    let res = contract
        .acl_add_admin_unchecked(Caller::Contract, role, account.id())
        .await?;
    assert_success_with(res, false);

    Ok(())
}

#[tokio::test]
async fn test_acl_revoke_admin() -> anyhow::Result<()> {
    let setup = Setup::new().await?;
    let role = "LevelB";
    let admin = setup.new_account_as_admin(&[role]).await?;

    setup
        .contract
        .assert_acl_is_admin(true, role, admin.id())
        .await;

    // Revoke is a no-op if revoker is not an admin for the role.
    let revoker = setup.new_account_as_admin(&[]).await?;
    let res = setup
        .contract
        .acl_revoke_admin(revoker.into(), role, admin.id())
        .await?;
    assert_eq!(res, None);
    let revoker = setup.new_account_as_admin(&["LevelA"]).await?;
    let res = setup
        .contract
        .acl_revoke_admin(revoker.into(), role, admin.id())
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
        .acl_revoke_admin(revoker.into(), role, admin.id())
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
        .acl_revoke_admin(revoker.into(), role, account.id())
        .await?;
    assert_eq!(res, Some(false));

    Ok(())
}

#[tokio::test]
async fn test_acl_renounce_admin() -> anyhow::Result<()> {
    let setup = Setup::new().await?;
    let role = "LevelC";

    // An account which is isn't admin calls `acl_renounce_admin`.
    let res = setup
        .contract
        .acl_renounce_admin(setup.account.clone().into(), role)
        .await?;
    assert_eq!(res, false);

    // An admin calls `acl_renounce_admin`.
    let admin = setup.new_account_as_admin(&[role]).await?;
    setup
        .contract
        .assert_acl_is_admin(true, role, admin.id())
        .await;
    let res = setup
        .contract
        .acl_renounce_admin(admin.clone().into(), role)
        .await?;
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
    let account = setup.new_account_as_admin(&["LevelA", "LevelC"]).await?;

    setup
        .contract
        .assert_acl_is_admin(true, "LevelA", account.id())
        .await;
    setup
        .contract
        .assert_acl_is_admin(true, "LevelC", account.id())
        .await;

    // Revoke admin permissions for one of the roles.
    let res = setup
        .contract
        .acl_revoke_admin_unchecked(Caller::Contract, "LevelA", account.id())
        .await?;
    assert_success_with(res, true);
    setup
        .contract
        .assert_acl_is_admin(false, "LevelA", account.id())
        .await;
    setup
        .contract
        .assert_acl_is_admin(true, "LevelC", account.id())
        .await;

    // Revoke admin permissions for the other role too.
    let res = setup
        .contract
        .acl_revoke_admin_unchecked(Caller::Contract, "LevelC", account.id())
        .await?;
    assert_success_with(res, true);
    setup
        .contract
        .assert_acl_is_admin(false, "LevelA", account.id())
        .await;
    setup
        .contract
        .assert_acl_is_admin(false, "LevelC", account.id())
        .await;

    // Revoking behaves as expected if the permission is not present.
    let res = setup
        .contract
        .acl_revoke_admin_unchecked(Caller::Contract, "LevelC", account.id())
        .await?;
    assert_success_with(res, false);

    Ok(())
}

#[tokio::test]
async fn test_acl_has_role() -> anyhow::Result<()> {
    let Setup {
        contract, account, ..
    } = Setup::new().await?;
    let role = "LevelA";

    let has_role = contract
        .acl_has_role(account.clone().into(), role, account.id())
        .await?;
    assert_eq!(has_role, false);

    contract
        .acl_grant_role_unchecked(Caller::Contract, role, account.id())
        .await?
        .into_result()?;

    let has_role = contract
        .acl_has_role(account.clone().into(), role, account.id())
        .await?;
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
    let role = "LevelB";

    let granter = account;
    let grantee = worker.dev_create_account().await?;

    // An account which isn't admin can't grant the role.
    contract
        .assert_acl_is_admin(false, role, granter.id())
        .await;
    let granted = contract
        .acl_grant_role(granter.clone().into(), role, grantee.id())
        .await?;
    assert_eq!(granted, None);
    contract
        .assert_acl_has_role(false, role, grantee.id())
        .await;

    // Admin can grant the role.
    contract
        .acl_add_admin_unchecked(Caller::Contract, role, granter.id())
        .await?
        .into_result()?;
    let granted = contract
        .acl_grant_role(granter.clone().into(), role, grantee.id())
        .await?;
    assert_eq!(granted, Some(true));
    contract.assert_acl_has_role(true, role, grantee.id()).await;

    // Granting the role to an account which already is a grantee.
    let granted = contract
        .acl_grant_role(granter.clone().into(), role, grantee.id())
        .await?;
    assert_eq!(granted, Some(false));

    Ok(())
}

#[tokio::test]
async fn test_acl_grant_role_unchecked() -> anyhow::Result<()> {
    let Setup {
        contract, account, ..
    } = Setup::new().await?;
    let role = "LevelA";

    contract
        .assert_acl_has_role(false, role, account.id())
        .await;
    let res = contract
        .acl_grant_role_unchecked(Caller::Contract, role, account.id())
        .await?;
    assert_success_with(res, true);
    contract.assert_acl_has_role(true, role, account.id()).await;

    // Granting a role again behaves as expected.
    let res = contract
        .acl_grant_role_unchecked(Caller::Contract, role, account.id())
        .await?;
    assert_success_with(res, false);

    Ok(())
}

#[tokio::test]
async fn test_acl_revoke_role() -> anyhow::Result<()> {
    let setup = Setup::new().await?;
    let role = "LevelB";
    let grantee = setup.new_account_with_roles(&[role]).await?;

    setup
        .contract
        .assert_acl_has_role(true, role, grantee.id())
        .await;

    // Revoke is a no-op if revoker is not an admin for the role.
    let revoker = setup.new_account_as_admin(&[]).await?;
    let res = setup
        .contract
        .acl_revoke_role(revoker.into(), role, grantee.id())
        .await?;
    assert_eq!(res, None);
    let revoker = setup.new_account_as_admin(&["LevelA"]).await?;
    let res = setup
        .contract
        .acl_revoke_role(revoker.into(), role, grantee.id())
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
        .acl_revoke_role(revoker.into(), role, grantee.id())
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
        .acl_revoke_role(revoker.into(), role, account.id())
        .await?;
    assert_eq!(res, Some(false));

    Ok(())
}

#[tokio::test]
async fn test_acl_renounce_role() -> anyhow::Result<()> {
    let setup = Setup::new().await?;
    let role = "LevelC";

    // An account which is isn't grantee calls `acl_renounce_role`.
    let res = setup
        .contract
        .acl_renounce_role(setup.account.clone().into(), role)
        .await?;
    assert_eq!(res, false);

    // A grantee calls `acl_renounce_admin`.
    let grantee = setup.new_account_with_roles(&[role]).await?;
    setup
        .contract
        .assert_acl_has_role(true, role, grantee.id())
        .await;
    let res = setup
        .contract
        .acl_renounce_role(grantee.clone().into(), role)
        .await?;
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
    let account = setup.new_account_with_roles(&["LevelA", "LevelC"]).await?;

    setup
        .contract
        .assert_acl_has_role(true, "LevelA", account.id())
        .await;
    setup
        .contract
        .assert_acl_has_role(true, "LevelC", account.id())
        .await;

    // Revoke one of the roles.
    let res = setup
        .contract
        .acl_revoke_role_unchecked(Caller::Contract, "LevelA", account.id())
        .await?;
    assert_success_with(res, true);
    setup
        .contract
        .assert_acl_has_role(false, "LevelA", account.id())
        .await;
    setup
        .contract
        .assert_acl_has_role(true, "LevelC", account.id())
        .await;

    // Revoke the other role too.
    let res = setup
        .contract
        .acl_revoke_role_unchecked(Caller::Contract, "LevelC", account.id())
        .await?;
    assert_success_with(res, true);
    setup
        .contract
        .assert_acl_has_role(false, "LevelA", account.id())
        .await;
    setup
        .contract
        .assert_acl_has_role(false, "LevelC", account.id())
        .await;

    // Revoking behaves as expected if the role is not granted.
    let res = setup
        .contract
        .acl_revoke_role_unchecked(Caller::Contract, "LevelC", account.id())
        .await?;
    assert_success_with(res, false);

    Ok(())
}

#[tokio::test]
async fn test_attribute_access_control_any() -> anyhow::Result<()> {
    let setup = Setup::new().await?;
    let raw_contract = setup.contract.contract();
    let expected_result = "hello world".to_string();

    // Account without any of the required permissions is restricted.
    let account = setup.new_account_with_roles(&[]).await?;
    call_restricted_greeting(raw_contract, &account)
        .await?
        .assert_acl_failure();
    let account = setup.new_account_with_roles(&["LevelB"]).await?;
    call_restricted_greeting(raw_contract, &account)
        .await?
        .assert_acl_failure();

    // A super-admin which has not been granted the role is restricted.
    let super_admin = setup.new_super_admin_account().await?;
    call_restricted_greeting(raw_contract, &super_admin)
        .await?
        .assert_acl_failure();

    // An admin for a permitted role is restricted (no grantee of role itself).
    let admin = setup.new_account_as_admin(&["LevelA"]).await?;
    call_restricted_greeting(raw_contract, &admin)
        .await?
        .assert_acl_failure();

    // Account with one of the required permissions succeeds.
    let account = setup.new_account_with_roles(&["LevelA"]).await?;
    call_restricted_greeting(raw_contract, &account)
        .await?
        .assert_success(expected_result.clone());
    let account = setup.new_account_with_roles(&["LevelC"]).await?;
    call_restricted_greeting(raw_contract, &account)
        .await?
        .assert_success(expected_result.clone());
    let account = setup.new_account_with_roles(&["LevelA", "LevelB"]).await?;
    call_restricted_greeting(raw_contract, &account)
        .await?
        .assert_success(expected_result.clone());

    // Account with both permissions succeeds.
    let account = setup.new_account_with_roles(&["LevelA", "LevelC"]).await?;
    call_restricted_greeting(raw_contract, &account)
        .await?
        .assert_success(expected_result.clone());
    let account = setup
        .new_account_with_roles(&["LevelA", "LevelB", "LevelC"])
        .await?;
    call_restricted_greeting(raw_contract, &account)
        .await?
        .assert_success(expected_result.clone());

    Ok(())
}

#[tokio::test]
async fn test_acl_init_super_admin_is_private() -> anyhow::Result<()> {
    let Setup {
        contract, account, ..
    } = Setup::new().await?;
    let res = contract
        .acl_init_super_admin(account.clone().into(), account.id())
        .await?;
    assert_private_method_failure(res, "acl_init_super_admin");
    Ok(())
}

#[tokio::test]
async fn test_acl_add_super_admin_unchecked_is_private() -> anyhow::Result<()> {
    let Setup {
        contract, account, ..
    } = Setup::new().await?;
    let res = contract
        .acl_add_super_admin_unchecked(account.clone().into(), account.id())
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
        .acl_revoke_super_admin_unchecked(account.clone().into(), account.id())
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
        .acl_add_admin_unchecked(account.clone().into(), "LevelA", account.id())
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
        .acl_revoke_admin_unchecked(account.clone().into(), "LevelA", account.id())
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
        .acl_grant_role_unchecked(account.clone().into(), "LevelA", account.id())
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
        .acl_revoke_role_unchecked(account.clone().into(), "LevelA", account.id())
        .await?;
    assert_private_method_failure(res, "acl_revoke_role_unchecked");
    Ok(())
}
