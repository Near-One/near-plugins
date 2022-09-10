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
    let contract = worker.dev_deploy(&wasm).await?;

    let account = worker.dev_create_account().await?;

    // TODO add helper functions to execute frequent transactions

    let res = account
        .call(contract.id(), "acl_has_role")
        .args_json(json!({
            "role": "Level1",
            "account_id": account.id(),
        }))
        .view()
        .await?;
    assert_eq!(res.json::<bool>()?, false);

    contract
        .call("acl_grant_role_unchecked")
        .args_json(json!({
            "role": "Level1",
            "account_id": account.id(),
        }))
        .max_gas()
        .transact()
        .await?
        .into_result()?;

    let res = account
        .call(contract.id(), "acl_has_role")
        .args_json(json!({
            "role": "Level1",
            "account_id": account.id(),
        }))
        .view()
        .await?;
    assert_eq!(res.json::<bool>()?, true);

    Ok(())
}
