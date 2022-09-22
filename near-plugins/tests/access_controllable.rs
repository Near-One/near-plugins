mod common;

use common::access_controllable_contract::{AccessControllableContract, Caller};
use common::utils::assert_private_method_failure;
use near_sdk::serde_json::json;
use workspaces::Account;

const PROJECT_PATH: &str = "./tests/contracts/access_controllable";

/// Bundles resources required in tests.
struct Setup {
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

        Ok(Self { contract, account })
    }
}

/// Smoke test of contract setup and basic functionality.
#[tokio::test]
async fn test_set_and_get_status() -> anyhow::Result<()> {
    let Setup { contract, account } = Setup::new().await?;
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
async fn test_acl_has_role() -> anyhow::Result<()> {
    let Setup { contract, account } = Setup::new().await?;
    let role = "LevelA";

    // Anyone may call `acl_has_role`.
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
async fn test_acl_grant_role_unchecked_is_private() -> anyhow::Result<()> {
    let Setup { contract, account } = Setup::new().await?;
    let res = contract
        .acl_grant_role_unchecked(account.clone().into(), "LevelA", account.id())
        .await?;
    assert_private_method_failure(res, "acl_grant_role_unchecked");
    Ok(())
}

#[tokio::test]
async fn test_acl_grant_role_unchecked() -> anyhow::Result<()> {
    let Setup { contract, account } = Setup::new().await?;
    let role = "LevelA";

    contract
        .assert_acl_has_role(false, role, account.id())
        .await;
    contract
        .acl_grant_role_unchecked(Caller::Contract, role, account.id())
        .await?
        .into_result()?;
    contract.assert_acl_has_role(true, role, account.id()).await;

    // Granting a role again doesn't lead to failures.
    contract
        .acl_grant_role_unchecked(Caller::Contract, role, account.id())
        .await?
        .into_result()?;

    Ok(())
}
