// Using `pub` to avoid invalid `dead_code` warnings, see
// https://users.rust-lang.org/t/invalid-dead-code-warning-for-submodule-in-integration-test/80259
pub mod common;

use anyhow::Ok;
use common::upgradable_contract::UpgradableContract;
use common::utils::{
    assert_failure_with, assert_only_owner_permission_failure, assert_success_with_unit_return,
    fast_forward_beyond, get_transaction_block, sdk_duration_from_secs,
};
use near_sdk::serde_json::json;
use near_sdk::{CryptoHash, Duration, Timestamp};
use std::path::Path;
use workspaces::network::Sandbox;
use workspaces::result::ExecutionFinalResult;
use workspaces::{Account, AccountId, Worker};

const PROJECT_PATH: &str = "./tests/contracts/upgradable";

const ERR_MSG_NO_STAGING_TS: &str = "Upgradable: staging timestamp isn't set";
const ERR_MSG_DEPLOY_CODE_TOO_EARLY: &str = "Upgradable: Deploy code too early: staging ends on";
const ERR_MSG_UPDATE_DURATION_TOO_EARLY: &str =
    "Upgradable: Update duration too early: staging ends on";

/// Allows spinning up a setup for testing the contract in [`PROJECT_PATH`] and bundles related
/// resources.
struct Setup {
    /// The worker interacting with the current sandbox.
    worker: Worker<Sandbox>,
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
    async fn new(
        worker: Worker<Sandbox>,
        owner: Option<AccountId>,
        staging_duration: Option<Duration>,
    ) -> anyhow::Result<Self> {
        // Compile and deploy the contract.
        let wasm = common::repo::compile_project(Path::new(PROJECT_PATH), "upgradable").await?;
        let contract = worker.dev_deploy(&wasm).await?;
        let upgradable_contract = UpgradableContract::new(contract.clone());

        // Call the contract's constructor.
        contract
            .call("new")
            .args_json(json!({
                "owner": owner,
                "staging_duration": staging_duration,
            }))
            .max_gas()
            .transact()
            .await?
            .into_result()?;

        let unauth_account = worker.dev_create_account().await?;
        Ok(Self {
            worker,
            upgradable_contract,
            unauth_account,
        })
    }

    /// Computes the expected staging timestamp based on the result of a transaction that calls a
    /// function which sets the timestamp. For example a transaction which calls
    /// `Upgradable::up_stage_code`. The function call is expected to be the first action in the
    /// transaction.
    ///
    /// Panics if the block timestamp cannot be retrieved.
    async fn expected_staging_timestamp(
        &self,
        result: ExecutionFinalResult,
        delay: Duration,
    ) -> Timestamp {
        // Grab the receipt corresponding to the function call.
        let receipt = result
            .receipt_outcomes()
            .get(0)
            .expect("There should be at least one receipt outcome");
        let block_timestamp = get_transaction_block(&self.worker, receipt)
            .await
            .expect("Should retrieve the transaction's block")
            .timestamp();
        block_timestamp + delay
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

    /// Asserts the staging duration of the `Upgradable` contract equals the `expected_duration`.
    async fn assert_staging_duration(&self, expected_duration: Option<Duration>) {
        let status = self
            .upgradable_contract
            .up_get_delay_status(&self.unauth_account)
            .await
            .expect("Call to up_get_delay_status should succeed");
        assert_eq!(status.staging_duration, expected_duration);
    }

    /// Asserts the staging timestamp of the `Upgradable` contract equals the `expected_timestamp`.
    async fn assert_staging_timestamp(&self, expected_timestamp: Option<Timestamp>) {
        let status = self
            .upgradable_contract
            .up_get_delay_status(&self.unauth_account)
            .await
            .expect("Call to up_get_delay_status should succeed");
        assert_eq!(status.staging_timestamp, expected_timestamp);
    }

    /// Asserts the staged new staging duration of the `Upgradable` contract equals the
    /// `expected_duration`.
    async fn assert_new_staging_duration(&self, expected_duration: Option<Duration>) {
        let status = self
            .upgradable_contract
            .up_get_delay_status(&self.unauth_account)
            .await
            .expect("Call to up_get_delay_status should succeed");
        assert_eq!(status.new_staging_duration, expected_duration);
    }

    /// Asserts the staging timestamp of the new duration of an `Upgradable` contract equals the
    /// `expected_timestamp`.
    async fn assert_new_duration_staging_timestamp(&self, expected_timestamp: Option<Timestamp>) {
        let status = self
            .upgradable_contract
            .up_get_delay_status(&self.unauth_account)
            .await
            .expect("Call to up_get_delay_status should succeed");
        assert_eq!(status.new_staging_duration_timestamp, expected_timestamp);
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
    let _ = Setup::new(worker, None, None).await?;

    Ok(())
}

#[tokio::test]
async fn test_stage_code_permission_failure() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let owner = worker.dev_create_account().await?;
    let setup = Setup::new(
        worker,
        Some(owner.id().clone()),
        Some(sdk_duration_from_secs(42)),
    )
    .await?;

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
    let setup = Setup::new(worker, Some(owner.id().clone()), None).await?;

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

    // Verify staging timestamp. The staging duration defaults to zero if not set.
    let staging_timestamp = setup.expected_staging_timestamp(res, 0).await;
    setup
        .assert_staging_timestamp(Some(staging_timestamp))
        .await;

    Ok(())
}

#[tokio::test]
async fn test_stage_code_with_delay() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let owner = worker.dev_create_account().await?;
    let staging_duration = sdk_duration_from_secs(42);
    let setup = Setup::new(worker, Some(owner.id().clone()), Some(staging_duration)).await?;

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

    // Verify staging timestamp.
    let staging_timestamp = setup
        .expected_staging_timestamp(res, staging_duration)
        .await;
    setup
        .assert_staging_timestamp(Some(staging_timestamp))
        .await;

    Ok(())
}

#[tokio::test]
async fn test_staging_empty_code_clears_storage() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let owner = worker.dev_create_account().await?;
    let setup = Setup::new(
        worker,
        Some(owner.id().clone()),
        Some(sdk_duration_from_secs(42)),
    )
    .await?;

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

    // Verify the staging timestamp was removed along with the staged code.
    setup.assert_staging_timestamp(None).await;

    Ok(())
}

#[tokio::test]
async fn test_staged_code() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let owner = worker.dev_create_account().await?;
    let setup = Setup::new(
        worker,
        Some(owner.id().clone()),
        Some(sdk_duration_from_secs(42)),
    )
    .await?;

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
    let setup = Setup::new(
        worker,
        Some(owner.id().clone()),
        Some(sdk_duration_from_secs(42)),
    )
    .await?;

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
    let setup = Setup::new(worker.clone(), Some(owner.id().clone()), None).await?;

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

/// TODO stage code that corresponds to a valid contract. Call a method of that contract to verify
/// the staged code was deployed.
#[tokio::test]
async fn test_deploy_code_with_delay() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let owner = worker.dev_create_account().await?;
    let staging_duration = sdk_duration_from_secs(3);
    let setup = Setup::new(
        worker.clone(),
        Some(owner.id().clone()),
        Some(staging_duration),
    )
    .await?;

    // Stage some code.
    let code = vec![1, 2, 3];
    let res = setup
        .upgradable_contract
        .up_stage_code(&owner, code.clone())
        .await?;
    assert_success_with_unit_return(res);
    setup.assert_staged_code(Some(code)).await;

    // Let the staging duration pass.
    fast_forward_beyond(&worker, staging_duration).await;

    // Deploy staged code.
    let res = setup.upgradable_contract.up_deploy_code(&owner).await?;
    assert_success_with_unit_return(res);

    Ok(())
}

#[tokio::test]
async fn test_deploy_code_with_delay_failure_too_early() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let owner = worker.dev_create_account().await?;
    let setup = Setup::new(
        worker.clone(),
        Some(owner.id().clone()),
        Some(sdk_duration_from_secs(1024)),
    )
    .await?;

    // Stage some code.
    let code = vec![1, 2, 3];
    let res = setup
        .upgradable_contract
        .up_stage_code(&owner, code.clone())
        .await?;
    assert_success_with_unit_return(res);
    setup.assert_staged_code(Some(code)).await;

    // Let some time pass but not enough.
    fast_forward_beyond(&worker, sdk_duration_from_secs(1)).await;

    // Verify trying to deploy staged code fails.
    let res = setup.upgradable_contract.up_deploy_code(&owner).await?;
    assert_failure_with(res, ERR_MSG_DEPLOY_CODE_TOO_EARLY);

    Ok(())
}

#[tokio::test]
async fn test_deploy_code_permission_failure() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let owner = worker.dev_create_account().await?;
    let setup = Setup::new(worker, Some(owner.id().clone()), None).await?;

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
    let setup = Setup::new(
        worker,
        Some(owner.id().clone()),
        Some(sdk_duration_from_secs(42)),
    )
    .await?;

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
    assert_failure_with(res, ERR_MSG_NO_STAGING_TS);

    Ok(())
}

#[tokio::test]
async fn test_init_staging_duration_permission_failure() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let owner = worker.dev_create_account().await?;
    let setup = Setup::new(worker, Some(owner.id().clone()), None).await?;

    let res = setup
        .upgradable_contract
        .up_init_staging_duration(&setup.unauth_account, sdk_duration_from_secs(23))
        .await?;
    assert_only_owner_permission_failure(res);

    setup.assert_staging_duration(None).await;

    Ok(())
}

#[tokio::test]
async fn test_init_staging_duration() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let owner = worker.dev_create_account().await?;
    let setup = Setup::new(worker, Some(owner.id().clone()), None).await?;

    // Verify the contract was initialized without staging duration.
    setup.assert_staging_duration(None).await;

    // Initialize the staging duration.
    let staging_duration = sdk_duration_from_secs(42);
    let res = setup
        .upgradable_contract
        .up_init_staging_duration(&owner, staging_duration)
        .await?;
    assert_success_with_unit_return(res.clone());

    // Verify the staging duration was set.
    setup.assert_staging_duration(Some(staging_duration)).await;

    Ok(())
}

#[tokio::test]
async fn test_stage_update_staging_duration_permission_failure() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let owner = worker.dev_create_account().await?;
    let staging_duration = sdk_duration_from_secs(42);
    let setup = Setup::new(worker, Some(owner.id().clone()), Some(staging_duration)).await?;

    let res = setup
        .upgradable_contract
        .up_stage_update_staging_duration(&setup.unauth_account, sdk_duration_from_secs(23))
        .await?;
    assert_only_owner_permission_failure(res);

    setup.assert_new_staging_duration(None).await;

    Ok(())
}

#[tokio::test]
async fn test_stage_update_staging_duration() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let owner = worker.dev_create_account().await?;
    let staging_duration = sdk_duration_from_secs(42);
    let setup = Setup::new(worker, Some(owner.id().clone()), Some(staging_duration)).await?;

    // Initially there's no new staging duration staged and no timestamp set.
    setup.assert_new_staging_duration(None).await;
    setup.assert_new_duration_staging_timestamp(None).await;

    // Stage a new duration.
    let new_staging_duration = sdk_duration_from_secs(23);
    let res = setup
        .upgradable_contract
        .up_stage_update_staging_duration(&owner, new_staging_duration)
        .await?;
    assert_success_with_unit_return(res.clone());

    // Verify the new duration was staged.
    setup
        .assert_new_staging_duration(Some(new_staging_duration))
        .await;

    // Verify timestamp for the staging duration update.
    let expected_timestamp = setup
        .expected_staging_timestamp(res, staging_duration)
        .await;
    setup
        .assert_new_duration_staging_timestamp(Some(expected_timestamp))
        .await;

    Ok(())
}

#[tokio::test]
async fn test_apply_update_staging_duration_permission_failure() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let owner = worker.dev_create_account().await?;
    let staging_duration = sdk_duration_from_secs(21);
    let setup = Setup::new(worker, Some(owner.id().clone()), Some(staging_duration)).await?;

    // Verify the initial staging duration.
    setup.assert_staging_duration(Some(staging_duration)).await;

    // Stage a new duration.
    let new_staging_duration = sdk_duration_from_secs(23);
    let res = setup
        .upgradable_contract
        .up_stage_update_staging_duration(&owner, new_staging_duration)
        .await?;
    assert_success_with_unit_return(res.clone());

    // Let the staging duration pass.
    fast_forward_beyond(&setup.worker, staging_duration).await;

    // Verify applying the update fails due to missing permission.
    let res = setup
        .upgradable_contract
        .up_apply_update_staging_duration(&setup.unauth_account)
        .await?;
    assert_only_owner_permission_failure(res);
    setup.assert_staging_duration(Some(staging_duration)).await;
    setup
        .assert_new_staging_duration(Some(new_staging_duration))
        .await;

    Ok(())
}

#[tokio::test]
async fn test_apply_update_staging_duration() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let owner = worker.dev_create_account().await?;
    let staging_duration = sdk_duration_from_secs(21);
    let setup = Setup::new(worker, Some(owner.id().clone()), Some(staging_duration)).await?;

    // Verify the initial staging duration.
    setup.assert_staging_duration(Some(staging_duration)).await;

    // Stage a new duration.
    let new_staging_duration = sdk_duration_from_secs(12);
    let res = setup
        .upgradable_contract
        .up_stage_update_staging_duration(&owner, new_staging_duration)
        .await?;
    assert_success_with_unit_return(res.clone());

    // Let the staging duration pass.
    fast_forward_beyond(&setup.worker, staging_duration).await;

    // Apply the update and verify the new duration was set.
    let res = setup
        .upgradable_contract
        .up_apply_update_staging_duration(&owner)
        .await?;
    assert_success_with_unit_return(res);
    setup
        .assert_staging_duration(Some(new_staging_duration))
        .await;

    Ok(())
}

#[tokio::test]
async fn test_apply_update_staging_duration_failure_too_early() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let owner = worker.dev_create_account().await?;
    let staging_duration = sdk_duration_from_secs(1024);
    let setup = Setup::new(worker, Some(owner.id().clone()), Some(staging_duration)).await?;

    // Verify the initial staging duration.
    setup.assert_staging_duration(Some(staging_duration)).await;

    // Stage a new duration.
    let new_staging_duration = sdk_duration_from_secs(42);
    let res = setup
        .upgradable_contract
        .up_stage_update_staging_duration(&owner, new_staging_duration)
        .await?;
    assert_success_with_unit_return(res.clone());

    // Let some time pass but not enough.
    fast_forward_beyond(&setup.worker, sdk_duration_from_secs(1)).await;

    // Verify trying to apply the new duration fails.
    let res = setup
        .upgradable_contract
        .up_apply_update_staging_duration(&owner)
        .await?;
    assert_failure_with(res, ERR_MSG_UPDATE_DURATION_TOO_EARLY);

    Ok(())
}
