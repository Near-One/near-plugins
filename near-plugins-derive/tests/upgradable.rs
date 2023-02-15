// Using `pub` to avoid invalid `dead_code` warnings, see
// https://users.rust-lang.org/t/invalid-dead-code-warning-for-submodule-in-integration-test/80259
pub mod common;

use anyhow::Ok;
use common::upgradable_contract::UpgradableContract;
use common::utils::{
    assert_failure_with, assert_only_owner_permission_failure, assert_success_with,
    assert_success_with_unit_return,
};
use near_sdk::serde_json::json;
use near_sdk::CryptoHash;
use std::path::Path;
use workspaces::network::Sandbox;
use workspaces::result::ExecutionFinalResult;
use workspaces::{Account, AccountId, Contract, Worker};

const PROJECT_PATH: &str = "./tests/contracts/upgradable";
const PROJECT_PATH_2: &str = "./tests/contracts/upgradable_2";

const ERR_MSG_NO_STAGED_CODE: &str = "Upgradable: No staged code";

/// Allows spinning up a setup for testing the contract in [`PROJECT_PATH`] and bundles related
/// resources.
struct Setup {
    /// A deployed instance of the contract.
    contract: Contract,
    /// Wrapper around the deployed contract that facilitates interacting with methods provided by
    /// the `Upgradable` plugin.
    upgradable_contract: UpgradableContract,
    /// A newly created account without any `Ownable` permissions.
    unauth_account: Account,
}

impl Setup {
    /// Deploys and initializes the contract in [`PROJECT_PATH`] and returns a new `Setup`.
    ///
    /// The `owner` and `staging_duration` parameters are passed to the contract's constructor,
    /// allowing to optionally set these values during initialization.
    async fn new(worker: Worker<Sandbox>, owner: Option<AccountId>) -> anyhow::Result<Self> {
        // Compile and deploy the contract.
        let wasm = common::repo::compile_project(Path::new(PROJECT_PATH), "upgradable").await?;
        let contract = worker.dev_deploy(&wasm).await?;
        let upgradable_contract = UpgradableContract::new(contract.clone());

        // Call the contract's constructor.
        contract
            .call("new")
            .args_json(json!({
                "owner": owner,
            }))
            .max_gas()
            .transact()
            .await?
            .into_result()?;

        let unauth_account = worker.dev_create_account().await?;
        Ok(Self {
            contract,
            upgradable_contract,
            unauth_account,
        })
    }

    /// Asserts staged code equals `expected_code`.
    async fn assert_staged_code(&self, expected_code: Option<Vec<u8>>) {
        let staged = self
            .upgradable_contract
            .up_staged_code(&self.unauth_account)
            .await
            .expect("Call to up_staged_code should succeed");
        assert_eq!(staged, expected_code);
    }
    async fn call_is_upgraded(&self, caller: &Account) -> workspaces::Result<ExecutionFinalResult> {
        // `is_upgraded` could be called via `view`, however here it is called via `transact` so we
        // get an `ExecutionFinalResult` that can be passed to `assert_*` methods from
        // `common::utils`. It is acceptable since all we care about is whether the method exists.
        caller
            .call(self.contract.id(), "is_upgraded")
            .max_gas()
            .transact()
            .await
    }
}

/// Panics if the conversion fails.
fn convert_code_to_crypto_hash(code: &[u8]) -> CryptoHash {
    near_sdk::env::sha256(code)
        .try_into()
        .expect("Code should be converted to CryptoHash")
}

/// Smoke test of contract setup.
#[tokio::test]
async fn test_setup() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let _ = Setup::new(worker, None).await?;

    Ok(())
}

#[tokio::test]
async fn test_stage_code_permission_failure() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let owner = worker.dev_create_account().await?;
    let setup = Setup::new(worker, Some(owner.id().clone())).await?;

    let res = setup
        .upgradable_contract
        .up_stage_code(&setup.unauth_account, vec![])
        .await?;
    assert_only_owner_permission_failure(res);

    setup.assert_staged_code(None).await;

    Ok(())
}

#[tokio::test]
async fn test_stage_code_without_delay() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let owner = worker.dev_create_account().await?;
    let setup = Setup::new(worker, Some(owner.id().clone())).await?;

    // Stage code.
    let code = vec![1, 2, 3];
    let res = setup
        .upgradable_contract
        .up_stage_code(&owner, code.clone())
        .await?;
    assert_success_with_unit_return(res.clone());

    // Verify code was staged.
    let staged = setup
        .upgradable_contract
        .up_staged_code(&setup.unauth_account)
        .await?
        .expect("Code should be staged");
    assert_eq!(staged, code);

    Ok(())
}

#[tokio::test]
async fn test_staging_empty_code_clears_storage() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let owner = worker.dev_create_account().await?;
    let setup = Setup::new(worker, Some(owner.id().clone())).await?;

    // First stage some code.
    let code = vec![1, 2, 3];
    let res = setup
        .upgradable_contract
        .up_stage_code(&owner, code.clone())
        .await?;
    assert_success_with_unit_return(res);
    setup.assert_staged_code(Some(code)).await;

    // Verify staging empty code removes it.
    let res = setup
        .upgradable_contract
        .up_stage_code(&owner, vec![])
        .await?;
    assert_success_with_unit_return(res);
    setup.assert_staged_code(None).await;

    Ok(())
}

#[tokio::test]
async fn test_staged_code() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let owner = worker.dev_create_account().await?;
    let setup = Setup::new(worker, Some(owner.id().clone())).await?;

    // No code staged.
    let staged = setup
        .upgradable_contract
        .up_staged_code(&setup.unauth_account)
        .await?;
    assert_eq!(staged, None);

    // Stage code.
    let code = vec![1, 2, 3];
    let res = setup
        .upgradable_contract
        .up_stage_code(&owner, code.clone())
        .await?;
    assert_success_with_unit_return(res);

    // Some code is staged.
    let staged = setup
        .upgradable_contract
        .up_staged_code(&setup.unauth_account)
        .await?
        .expect("Code should be staged");
    assert_eq!(staged, code);

    Ok(())
}

#[tokio::test]
async fn test_staged_code_hash() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let owner = worker.dev_create_account().await?;
    let setup = Setup::new(worker, Some(owner.id().clone())).await?;

    // No code staged.
    let staged_hash = setup
        .upgradable_contract
        .up_staged_code_hash(&setup.unauth_account)
        .await?;
    assert_eq!(staged_hash, None);

    // Stage code.
    let code = vec![1, 2, 3];
    let res = setup
        .upgradable_contract
        .up_stage_code(&owner, code.clone())
        .await?;
    assert_success_with_unit_return(res);

    // Some code is staged.
    let staged_hash = setup
        .upgradable_contract
        .up_staged_code_hash(&setup.unauth_account)
        .await?
        .expect("Code should be staged");
    let code_hash = convert_code_to_crypto_hash(code.as_slice());
    assert_eq!(staged_hash, code_hash);

    Ok(())
}

#[tokio::test]
async fn test_deploy_code_without_delay() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let owner = worker.dev_create_account().await?;
    let setup = Setup::new(worker.clone(), Some(owner.id().clone())).await?;

    // Stage some code.
    let code = vec![1, 2, 3];
    let res = setup
        .upgradable_contract
        .up_stage_code(&owner, code.clone())
        .await?;
    assert_success_with_unit_return(res);
    setup.assert_staged_code(Some(code)).await;

    // Deploy staged code.
    let res = setup.upgradable_contract.up_deploy_code(&owner).await?;
    assert_success_with_unit_return(res);

    Ok(())
}

/// Verifies the upgrade was successful by calling a method that's available only on the upgraded
/// contract. Ensures the new contract can be deployed and state migration succeeds.
#[tokio::test]
async fn test_deploy_code_and_call_method() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let owner = worker.dev_create_account().await?;
    let setup = Setup::new(worker.clone(), Some(owner.id().clone())).await?;

    // Verify function `is_upgraded` is not defined in the initial contract.
    let res = setup.call_is_upgraded(&setup.unauth_account).await?;
    assert_failure_with(res, "Action #0: MethodResolveError(MethodNotFound)");

    // Compile the other version of the contract and stage its code.
    let code = common::repo::compile_project(Path::new(PROJECT_PATH_2), "upgradable_2").await?;
    let res = setup
        .upgradable_contract
        .up_stage_code(&owner, code.clone())
        .await?;
    assert_success_with_unit_return(res);
    setup.assert_staged_code(Some(code)).await;

    // Deploy staged code.
    let res = setup.upgradable_contract.up_deploy_code(&owner).await?;
    assert_success_with_unit_return(res);

    // The newly deployed contract defines the function `is_upgraded`. Calling it successfully
    // verifies the staged contract is deployed and there are no issues with state migration.
    let res = setup.call_is_upgraded(&setup.unauth_account).await?;
    assert_success_with(res, true);

    Ok(())
}

#[tokio::test]
async fn test_deploy_code_permission_failure() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let owner = worker.dev_create_account().await?;
    let setup = Setup::new(worker, Some(owner.id().clone())).await?;

    let res = setup
        .upgradable_contract
        .up_deploy_code(&setup.unauth_account)
        .await?;
    assert_only_owner_permission_failure(res);

    Ok(())
}

/// `up_deploy_code` fails if there's no code staged.
#[tokio::test]
async fn test_deploy_code_empty_failure() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let owner = worker.dev_create_account().await?;
    let setup = Setup::new(worker, Some(owner.id().clone())).await?;

    // Verify there is no code staged.
    let staged_hash = setup
        .upgradable_contract
        .up_staged_code_hash(&setup.unauth_account)
        .await?;
    assert_eq!(staged_hash, None);

    // Verify failure of `up_deploy_code`.
    //
    // The staging timestamp is set when staging code and removed when unstaging code. So when there
    // is no code staged, there is no staging timestamp. Hence the error message regarding a missing
    // staging timestamp is expected.
    let res = setup.upgradable_contract.up_deploy_code(&owner).await?;
    assert_failure_with(res, ERR_MSG_NO_STAGED_CODE);

    Ok(())
}
