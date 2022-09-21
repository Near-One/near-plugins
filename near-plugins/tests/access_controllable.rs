mod common;

use common::access_controllable_contract::{AccessControllableContract, Caller};
use near_sdk::serde_json::json;

/// Smoke test of contract setup and basic functionality.
#[tokio::test]
async fn test_set_and_get_status() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let wasm = workspaces::compile_project("./tests/contracts/access_controllable").await?;
    let contract = worker.dev_deploy(&wasm).await?;

    let account = worker.dev_create_account().await?;
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
    let worker = workspaces::sandbox().await?;
    let wasm = workspaces::compile_project("./tests/contracts/access_controllable").await?;
    let contract = AccessControllableContract::new(worker.dev_deploy(&wasm).await?);
    let account = worker.dev_create_account().await?;

    let has_role = contract
        .acl_has_role(account.clone().into(), "LevelA", account.id())
        .await?;
    assert_eq!(has_role, false);

    contract
        .acl_grant_role_unchecked(Caller::Contract, "LevelA", account.id())
        .await?
        .into_result()?;

    let has_role = contract
        .acl_has_role(account.clone().into(), "LevelA", account.id())
        .await?;
    assert_eq!(has_role, true);

    Ok(())
}
