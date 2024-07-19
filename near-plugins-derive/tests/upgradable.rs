// Using `pub` to avoid invalid `dead_code` warnings, see
// https://users.rust-lang.org/t/invalid-dead-code-warning-for-submodule-in-integration-test/80259
pub mod common;

use anyhow::Ok;
use common::access_controllable_contract::AccessControllableContract;
use common::upgradable_contract::UpgradableContract;
use common::utils::{
    assert_failure_with, assert_insufficient_acl_permissions, assert_method_not_found_failure,
    assert_success_with, assert_success_with_unit_return, fast_forward_beyond,
    get_transaction_block, sdk_duration_from_secs,
};
use near_plugins::upgradable::FunctionCallArgs;
use near_sdk::serde_json::json;
use near_sdk::{CryptoHash, Duration, Gas, NearToken, Timestamp};
use near_workspaces::network::Sandbox;
use near_workspaces::result::ExecutionFinalResult;
use near_workspaces::{Account, AccountId, Contract, Worker};
use std::path::Path;

const PROJECT_PATH: &str = "./tests/contracts/upgradable";
const PROJECT_PATH_2: &str = "./tests/contracts/upgradable_2";
const PROJECT_PATH_STATE_MIGRATION: &str = "./tests/contracts/upgradable_state_migration";

const ERR_MSG_NO_STAGING_TS: &str = "Upgradable: staging timestamp isn't set";
const ERR_MSG_DEPLOY_CODE_TOO_EARLY: &str = "Upgradable: Deploy code too early: staging ends on";
const ERR_MSG_UPDATE_DURATION_TOO_EARLY: &str =
    "Upgradable: Update duration too early: staging ends on";

/// Allows spinning up a setup for testing the contract in [`PROJECT_PATH`] and bundles related
/// resources.
struct Setup {
    /// The worker interacting with the current sandbox.
    worker: Worker<Sandbox>,
    /// A deployed instance of the contract.
    contract: Contract,
    /// Wrapper around the deployed contract that facilitates interacting with methods provided by
    /// the `Upgradable` plugin.
    upgradable_contract: UpgradableContract,
    /// Wrapper around the deployed contract that facilitates interacting with methods provided by
    /// the `AccessControllable` plugin.
    acl_contract: AccessControllableContract,
    /// A newly created account without any `AccessControllable` permissions.
    unauth_account: Account,
}

impl Setup {
    /// Deploys and initializes the test contract in [`PROJECT_PATH`] and returns a new `Setup`.
    ///
    /// The `dao` and `staging_duration` parameters are passed to the contract's constructor,
    /// allowing to optionally grant the `DAO` role and initialize the staging duration.
    ///
    /// Grantees of the `DAO` role are authorized to call all protected `Upgradable` methods of the
    /// test contract, which facilitates testing.
    async fn new(
        worker: Worker<Sandbox>,
        dao: Option<AccountId>,
        staging_duration: Option<Duration>,
    ) -> anyhow::Result<Self> {
        // Compile and deploy the contract.
        let wasm = common::repo::compile_project(Path::new(PROJECT_PATH), "upgradable").await?;
        let contract = worker.dev_deploy(&wasm).await?;
        let upgradable_contract = UpgradableContract::new(contract.clone());
        let acl_contract = AccessControllableContract::new(contract.clone());

        // Call the contract's constructor.
        contract
            .call("new")
            .args_json(json!({
                "dao": dao,
                "staging_duration": staging_duration,
            }))
            .max_gas()
            .transact()
            .await?
            .into_result()?;

        let unauth_account = worker.dev_create_account().await?;
        Ok(Self {
            worker,
            contract,
            upgradable_contract,
            acl_contract,
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
            .first()
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

    async fn call_is_upgraded(
        &self,
        caller: &Account,
    ) -> near_workspaces::Result<ExecutionFinalResult> {
        // `is_upgraded` could be called via `view`, however here it is called via `transact` so we
        // get an `ExecutionFinalResult` that can be passed to `assert_*` methods from
        // `common::utils`. It is acceptable since all we care about is whether the method exists.
        caller
            .call(self.contract.id(), "is_upgraded")
            .max_gas()
            .transact()
            .await
    }

    async fn call_is_migrated(
        &self,
        caller: &Account,
    ) -> near_workspaces::Result<ExecutionFinalResult> {
        // `is_migrated` could be called via `view`, however here it is called via `transact` so we
        // get an `ExecutionFinalResult` that can be passed to `assert_*` methods from
        // `common::utils`. It is acceptable since all we care about is whether the method exists
        // and can be called successfully.
        caller
            .call(self.contract.id(), "is_migrated")
            .max_gas()
            .transact()
            .await
    }

    /// Calls the contract's `is_set_up` method and asserts it returns `true`. Panics on failure.
    async fn assert_is_set_up(&self, caller: &Account) {
        let res = caller
            .call(self.contract.id(), "is_set_up")
            .view()
            .await
            .expect("Function call should succeed");
        let is_set_up = res
            .json::<bool>()
            .expect("Should be able to deserialize the result");
        assert!(is_set_up);
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
    let worker = near_workspaces::sandbox().await?;
    let setup = Setup::new(worker, None, None).await?;
    setup.assert_is_set_up(&setup.unauth_account).await;

    Ok(())
}

#[tokio::test]
async fn test_stage_code_permission_failure() -> anyhow::Result<()> {
    let worker = near_workspaces::sandbox().await?;
    let dao = worker.dev_create_account().await?;
    let setup = Setup::new(
        worker,
        Some(dao.id().clone()),
        Some(sdk_duration_from_secs(42)),
    )
    .await?;

    // Only the roles passed as `code_stagers` to the `Upgradable` derive macro may successfully
    // call this method.
    let res = setup
        .upgradable_contract
        .up_stage_code(&setup.unauth_account, vec![])
        .await?;
    assert_insufficient_acl_permissions(
        res,
        "up_stage_code",
        &["CodeStager".to_string(), "DAO".to_string()],
    );

    // Verify no code was staged.
    setup.assert_staged_code(None).await;

    Ok(())
}

#[tokio::test]
async fn test_stage_code_without_delay() -> anyhow::Result<()> {
    let worker = near_workspaces::sandbox().await?;
    let dao = worker.dev_create_account().await?;
    let setup = Setup::new(worker, Some(dao.id().clone()), None).await?;

    // Stage code.
    let code = vec![1, 2, 3];
    let res = setup
        .upgradable_contract
        .up_stage_code(&dao, code.clone())
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
    let worker = near_workspaces::sandbox().await?;
    let dao = worker.dev_create_account().await?;
    let staging_duration = sdk_duration_from_secs(42);
    let setup = Setup::new(worker, Some(dao.id().clone()), Some(staging_duration)).await?;

    // Stage code.
    let code = vec![1, 2, 3];
    let res = setup
        .upgradable_contract
        .up_stage_code(&dao, code.clone())
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
    let worker = near_workspaces::sandbox().await?;
    let dao = worker.dev_create_account().await?;
    let setup = Setup::new(
        worker,
        Some(dao.id().clone()),
        Some(sdk_duration_from_secs(42)),
    )
    .await?;

    // First stage some code.
    let code = vec![1, 2, 3];
    let res = setup
        .upgradable_contract
        .up_stage_code(&dao, code.clone())
        .await?;
    assert_success_with_unit_return(res);
    setup.assert_staged_code(Some(code)).await;

    // Verify staging empty code removes it.
    let res = setup
        .upgradable_contract
        .up_stage_code(&dao, vec![])
        .await?;
    assert_success_with_unit_return(res);
    setup.assert_staged_code(None).await;

    // Verify the staging timestamp was removed along with the staged code.
    setup.assert_staging_timestamp(None).await;

    Ok(())
}

#[tokio::test]
async fn test_staged_code() -> anyhow::Result<()> {
    let worker = near_workspaces::sandbox().await?;
    let dao = worker.dev_create_account().await?;
    let setup = Setup::new(
        worker,
        Some(dao.id().clone()),
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
        .up_stage_code(&dao, code.clone())
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
    let worker = near_workspaces::sandbox().await?;
    let dao = worker.dev_create_account().await?;
    let setup = Setup::new(
        worker,
        Some(dao.id().clone()),
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
        .up_stage_code(&dao, code.clone())
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
    let worker = near_workspaces::sandbox().await?;
    let dao = worker.dev_create_account().await?;
    let setup = Setup::new(worker.clone(), Some(dao.id().clone()), None).await?;

    // Stage some code.
    let code = vec![1, 2, 3];
    let res = setup
        .upgradable_contract
        .up_stage_code(&dao, code.clone())
        .await?;
    assert_success_with_unit_return(res);
    setup.assert_staged_code(Some(code)).await;

    // Deploy staged code.
    let res = setup.upgradable_contract.up_deploy_code(&dao, None).await?;
    assert_success_with_unit_return(res);

    Ok(())
}

/// Verifies the upgrade was successful by calling a method that's available only on the upgraded
/// contract. Ensures the new contract can be deployed and state remains valid without
/// explicit state migration.
#[tokio::test]
async fn test_deploy_code_and_call_method() -> anyhow::Result<()> {
    let worker = near_workspaces::sandbox().await?;
    let dao = worker.dev_create_account().await?;
    let setup = Setup::new(worker.clone(), Some(dao.id().clone()), None).await?;

    // Verify function `is_upgraded` is not defined in the initial contract.
    let res = setup.call_is_upgraded(&setup.unauth_account).await?;
    assert_method_not_found_failure(res);

    // Compile the other version of the contract and stage its code.
    let code = common::repo::compile_project(Path::new(PROJECT_PATH_2), "upgradable_2").await?;
    let res = setup
        .upgradable_contract
        .up_stage_code(&dao, code.clone())
        .await?;
    assert_success_with_unit_return(res);
    setup.assert_staged_code(Some(code)).await;

    // Deploy staged code.
    let res = setup.upgradable_contract.up_deploy_code(&dao, None).await?;
    assert_success_with_unit_return(res);

    // The newly deployed contract defines the function `is_upgraded`. Calling it successfully
    // verifies the staged contract is deployed and there are no issues with state migration.
    let res = setup.call_is_upgraded(&setup.unauth_account).await?;
    assert_success_with(res, true);

    Ok(())
}

/// Deploys a new version of the contract that requires state migration and verifies the migration
/// succeeded.
#[tokio::test]
async fn test_deploy_code_with_migration() -> anyhow::Result<()> {
    let worker = near_workspaces::sandbox().await?;
    let dao = worker.dev_create_account().await?;
    let setup = Setup::new(worker.clone(), Some(dao.id().clone()), None).await?;

    // Verify function `is_migrated` is not defined in the initial contract.
    let res = setup.call_is_migrated(&setup.unauth_account).await?;
    assert_method_not_found_failure(res);

    // Compile the other version of the contract and stage its code.
    let code = common::repo::compile_project(
        Path::new(PROJECT_PATH_STATE_MIGRATION),
        "upgradable_state_migration",
    )
    .await?;
    let res = setup
        .upgradable_contract
        .up_stage_code(&dao, code.clone())
        .await?;
    assert_success_with_unit_return(res);
    setup.assert_staged_code(Some(code)).await;

    // Deploy staged code and call the new contract's `migrate` method.
    let function_call_args = FunctionCallArgs {
        function_name: "migrate".to_string(),
        arguments: Vec::new(),
        amount: NearToken::from_yoctonear(0),
        gas: Gas::from_tgas(3),
    };
    let res = setup
        .upgradable_contract
        .up_deploy_code(&dao, Some(function_call_args))
        .await?;
    assert_success_with_unit_return(res);

    // The newly deployed contract defines the function `is_migrated`. Calling it successfully
    // verifies the staged contract is deployed and state migration succeeded.
    let res = setup.call_is_migrated(&setup.unauth_account).await?;
    assert_success_with(res, true);

    Ok(())
}

/// Deploys a new version of the contract and, batched with the `DeployContractAction`, calls a
/// migration method that fails. Verifies the failure rolls back the deployment, i.e. the initial
/// code remains active.
#[tokio::test]
async fn test_deploy_code_with_migration_failure_rollback() -> anyhow::Result<()> {
    let worker = near_workspaces::sandbox().await?;
    let dao = worker.dev_create_account().await?;
    let setup = Setup::new(worker.clone(), Some(dao.id().clone()), None).await?;

    // Compile the other version of the contract and stage its code.
    let code = common::repo::compile_project(
        Path::new(PROJECT_PATH_STATE_MIGRATION),
        "upgradable_state_migration",
    )
    .await?;
    let res = setup
        .upgradable_contract
        .up_stage_code(&dao, code.clone())
        .await?;
    assert_success_with_unit_return(res);
    setup.assert_staged_code(Some(code)).await;

    // Deploy staged code and call the new contract's `migrate_with_failure` method.
    let function_call_args = FunctionCallArgs {
        function_name: "migrate_with_failure".to_string(),
        arguments: Vec::new(),
        amount: NearToken::from_yoctonear(0),
        gas: Gas::from_tgas(2),
    };
    let res = setup
        .upgradable_contract
        .up_deploy_code(&dao, Some(function_call_args))
        .await?;
    assert_failure_with(res, "Failing migration on purpose");

    // Verify `code` wasn't deployed by calling a function that is defined only in the initial
    // contract but not in the contract corresponding to the `code`.
    setup.assert_is_set_up(&setup.unauth_account).await;

    Ok(())
}

/// Deploys staged code in a batch transaction with two function call actions:
///
/// 1. `up_deploy_code` with a function call to a migration method that fails
/// 2. `up_stage_code` to remove staged code from storage
///
/// The pitfall is that a failure in the promise returned by 1 does _not_ make the transaction fail
/// and 2 executes anyway.
#[tokio::test]
async fn test_deploy_code_in_batch_transaction_pitfall() -> anyhow::Result<()> {
    let worker = near_workspaces::sandbox().await?;
    let dao = worker.dev_create_account().await?;
    let setup = Setup::new(worker.clone(), Some(dao.id().clone()), None).await?;

    // Compile the other version of the contract and stage its code.
    let code = common::repo::compile_project(
        Path::new(PROJECT_PATH_STATE_MIGRATION),
        "upgradable_state_migration",
    )
    .await?;
    let res = setup
        .upgradable_contract
        .up_stage_code(&dao, code.clone())
        .await?;
    assert_success_with_unit_return(res);
    setup.assert_staged_code(Some(code)).await;

    // Construct the function call actions to be executed in a batch transaction.
    // Note that we are attaching a call to `migrate_with_failure`, which will fail.
    let fn_call_deploy = near_workspaces::operations::Function::new("up_deploy_code")
        .args_json(json!({ "function_call_args": FunctionCallArgs {
        function_name: "migrate_with_failure".to_string(),
        arguments: Vec::new(),
        amount: NearToken::from_yoctonear(0),
        gas: Gas::from_tgas(2),
    } }))
        .gas(Gas::from_tgas(201));
    let fn_call_remove_code = near_workspaces::operations::Function::new("up_stage_code")
        .args_borsh(Vec::<u8>::new())
        .gas(Gas::from_tgas(90));

    let res = dao
        .batch(setup.contract.id())
        .call(fn_call_deploy)
        .call(fn_call_remove_code)
        .transact()
        .await?;

    // Here is the pitfall: Despite the failure of `migrate_with_failure`, the transaction succeeds.
    // This is due to `fn_call_deploy` _successfully_ returning a promise `p`. The promise `p`
    // fails, however that does not affect the result of the transaction.
    assert_success_with_unit_return(res.clone());

    // Verify the promise resulting from `fn_call_deploy` failed. There seems to be no public API to
    // get the status of an `ExecutionOutcome`, hence `is_failure` is used in combination with debug
    // formatting. Since this is test code we can use debug formatting for this purpose.
    let fn_call_deploy_receipt = res
        .receipt_outcomes()
        .get(1)
        .expect("There should be at least two receipts");
    assert!(fn_call_deploy_receipt.is_failure());
    assert!(format!("{fn_call_deploy_receipt:?}").contains("Failing migration on purpose"));

    // Verify `code` wasn't deployed by calling a function that is defined only in the initial
    // contract but not in the contract corresponding to `code`.
    setup.assert_is_set_up(&setup.unauth_account).await;

    // However the staged code was removed, i.e. `fn_call_remove_code` was executed anyway.
    setup.assert_staged_code(None).await;

    Ok(())
}

#[tokio::test]
async fn test_deploy_code_with_delay() -> anyhow::Result<()> {
    let worker = near_workspaces::sandbox().await?;
    let dao = worker.dev_create_account().await?;
    let staging_duration = sdk_duration_from_secs(3);
    let setup = Setup::new(
        worker.clone(),
        Some(dao.id().clone()),
        Some(staging_duration),
    )
    .await?;

    // Stage some code.
    let code = vec![1, 2, 3];
    let res = setup
        .upgradable_contract
        .up_stage_code(&dao, code.clone())
        .await?;
    assert_success_with_unit_return(res);
    setup.assert_staged_code(Some(code)).await;

    // Let the staging duration pass.
    fast_forward_beyond(&worker, staging_duration).await;

    // Deploy staged code.
    let res = setup.upgradable_contract.up_deploy_code(&dao, None).await?;
    assert_success_with_unit_return(res);

    Ok(())
}

#[tokio::test]
async fn test_deploy_code_with_delay_failure_too_early() -> anyhow::Result<()> {
    let worker = near_workspaces::sandbox().await?;
    let dao = worker.dev_create_account().await?;
    let setup = Setup::new(
        worker.clone(),
        Some(dao.id().clone()),
        Some(sdk_duration_from_secs(1024)),
    )
    .await?;

    // Stage some code.
    let code = vec![1, 2, 3];
    let res = setup
        .upgradable_contract
        .up_stage_code(&dao, code.clone())
        .await?;
    assert_success_with_unit_return(res);
    setup.assert_staged_code(Some(code)).await;

    // Let some time pass but not enough.
    fast_forward_beyond(&worker, sdk_duration_from_secs(1)).await;

    // Verify trying to deploy staged code fails.
    let res = setup.upgradable_contract.up_deploy_code(&dao, None).await?;
    assert_failure_with(res, ERR_MSG_DEPLOY_CODE_TOO_EARLY);

    // Verify `code` wasn't deployed by calling a function that is defined only in the initial
    // contract but not in the contract contract corresponding to `code`.
    setup.assert_is_set_up(&setup.unauth_account).await;

    Ok(())
}

#[tokio::test]
async fn test_deploy_code_permission_failure() -> anyhow::Result<()> {
    let worker = near_workspaces::sandbox().await?;
    let dao = worker.dev_create_account().await?;
    let setup = Setup::new(worker, Some(dao.id().clone()), None).await?;

    // Stage some code.
    let code = vec![1, 2, 3];
    let res = setup
        .upgradable_contract
        .up_stage_code(&dao, code.clone())
        .await?;
    assert_success_with_unit_return(res);
    setup.assert_staged_code(Some(code)).await;

    // Only the roles passed as `code_deployers` to the `Upgradable` derive macro may successfully
    // call this method.
    let res = setup
        .upgradable_contract
        .up_deploy_code(&setup.unauth_account, None)
        .await?;
    assert_insufficient_acl_permissions(
        res,
        "up_deploy_code",
        &["CodeDeployer".to_string(), "DAO".to_string()],
    );

    // Verify `code` wasn't deployed by calling a function that is defined only in the initial
    // contract but not in the contract contract corresponding to `code`.
    setup.assert_is_set_up(&setup.unauth_account).await;

    Ok(())
}

/// `up_deploy_code` fails if there's no code staged.
#[tokio::test]
async fn test_deploy_code_empty_failure() -> anyhow::Result<()> {
    let worker = near_workspaces::sandbox().await?;
    let dao = worker.dev_create_account().await?;
    let setup = Setup::new(
        worker,
        Some(dao.id().clone()),
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
    let res = setup.upgradable_contract.up_deploy_code(&dao, None).await?;
    assert_failure_with(res, ERR_MSG_NO_STAGING_TS);

    Ok(())
}

#[tokio::test]
async fn test_init_staging_duration_permission_failure() -> anyhow::Result<()> {
    let worker = near_workspaces::sandbox().await?;
    let dao = worker.dev_create_account().await?;
    let setup = Setup::new(worker, Some(dao.id().clone()), None).await?;

    // Only the roles passed as `duration_initializers` to the `Upgradable` derive macro may
    // successfully call this method.
    let res = setup
        .upgradable_contract
        .up_init_staging_duration(&setup.unauth_account, sdk_duration_from_secs(23))
        .await?;
    assert_insufficient_acl_permissions(
        res,
        "up_init_staging_duration",
        &["DurationManager".to_string(), "DAO".to_string()],
    );

    setup.assert_staging_duration(None).await;

    Ok(())
}

#[tokio::test]
async fn test_init_staging_duration() -> anyhow::Result<()> {
    let worker = near_workspaces::sandbox().await?;
    let dao = worker.dev_create_account().await?;
    let setup = Setup::new(worker, Some(dao.id().clone()), None).await?;

    // Verify the contract was initialized without staging duration.
    setup.assert_staging_duration(None).await;

    // Initialize the staging duration.
    let staging_duration = sdk_duration_from_secs(42);
    let res = setup
        .upgradable_contract
        .up_init_staging_duration(&dao, staging_duration)
        .await?;
    assert_success_with_unit_return(res.clone());

    // Verify the staging duration was set.
    setup.assert_staging_duration(Some(staging_duration)).await;

    Ok(())
}

#[tokio::test]
async fn test_stage_update_staging_duration_permission_failure() -> anyhow::Result<()> {
    let worker = near_workspaces::sandbox().await?;
    let dao = worker.dev_create_account().await?;
    let staging_duration = sdk_duration_from_secs(42);
    let setup = Setup::new(worker, Some(dao.id().clone()), Some(staging_duration)).await?;

    // Only the roles passed as `duration_update_stagers` to the `Upgradable` derive macro may
    // successfully call this method.
    let res = setup
        .upgradable_contract
        .up_stage_update_staging_duration(&setup.unauth_account, sdk_duration_from_secs(23))
        .await?;
    assert_insufficient_acl_permissions(
        res,
        "up_stage_update_staging_duration",
        &["DurationManager".to_string(), "DAO".to_string()],
    );

    // Verify no duration was staged.
    setup.assert_new_staging_duration(None).await;

    Ok(())
}

#[tokio::test]
async fn test_stage_update_staging_duration() -> anyhow::Result<()> {
    let worker = near_workspaces::sandbox().await?;
    let dao = worker.dev_create_account().await?;
    let staging_duration = sdk_duration_from_secs(42);
    let setup = Setup::new(worker, Some(dao.id().clone()), Some(staging_duration)).await?;

    // Initially there's no new staging duration staged and no timestamp set.
    setup.assert_new_staging_duration(None).await;
    setup.assert_new_duration_staging_timestamp(None).await;

    // Stage a new duration.
    let new_staging_duration = sdk_duration_from_secs(23);
    let res = setup
        .upgradable_contract
        .up_stage_update_staging_duration(&dao, new_staging_duration)
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
    let worker = near_workspaces::sandbox().await?;
    let dao = worker.dev_create_account().await?;
    let staging_duration = sdk_duration_from_secs(21);
    let setup = Setup::new(worker, Some(dao.id().clone()), Some(staging_duration)).await?;

    // Verify the initial staging duration.
    setup.assert_staging_duration(Some(staging_duration)).await;

    // Stage a new duration.
    let new_staging_duration = sdk_duration_from_secs(23);
    let res = setup
        .upgradable_contract
        .up_stage_update_staging_duration(&dao, new_staging_duration)
        .await?;
    assert_success_with_unit_return(res.clone());

    // Let the staging duration pass.
    fast_forward_beyond(&setup.worker, staging_duration).await;

    // Only the roles passed as `duration_update_appliers` to the `Upgradable` derive macro may
    // successfully call this method.
    let res = setup
        .upgradable_contract
        .up_apply_update_staging_duration(&setup.unauth_account)
        .await?;
    assert_insufficient_acl_permissions(
        res,
        "up_apply_update_staging_duration",
        &["DurationManager".to_string(), "DAO".to_string()],
    );

    // Verify the update was not applied.
    setup.assert_staging_duration(Some(staging_duration)).await;
    setup
        .assert_new_staging_duration(Some(new_staging_duration))
        .await;

    Ok(())
}

#[tokio::test]
async fn test_apply_update_staging_duration() -> anyhow::Result<()> {
    let worker = near_workspaces::sandbox().await?;
    let dao = worker.dev_create_account().await?;
    let staging_duration = sdk_duration_from_secs(21);
    let setup = Setup::new(worker, Some(dao.id().clone()), Some(staging_duration)).await?;

    // Verify the initial staging duration.
    setup.assert_staging_duration(Some(staging_duration)).await;

    // Stage a new duration.
    let new_staging_duration = sdk_duration_from_secs(12);
    let res = setup
        .upgradable_contract
        .up_stage_update_staging_duration(&dao, new_staging_duration)
        .await?;
    assert_success_with_unit_return(res.clone());

    // Let the staging duration pass.
    fast_forward_beyond(&setup.worker, staging_duration).await;

    // Apply the update and verify the new duration was set.
    let res = setup
        .upgradable_contract
        .up_apply_update_staging_duration(&dao)
        .await?;
    assert_success_with_unit_return(res);
    setup
        .assert_staging_duration(Some(new_staging_duration))
        .await;

    Ok(())
}

#[tokio::test]
async fn test_apply_update_staging_duration_failure_too_early() -> anyhow::Result<()> {
    let worker = near_workspaces::sandbox().await?;
    let dao = worker.dev_create_account().await?;
    let staging_duration = sdk_duration_from_secs(1024);
    let setup = Setup::new(worker, Some(dao.id().clone()), Some(staging_duration)).await?;

    // Verify the initial staging duration.
    setup.assert_staging_duration(Some(staging_duration)).await;

    // Stage a new duration.
    let new_staging_duration = sdk_duration_from_secs(42);
    let res = setup
        .upgradable_contract
        .up_stage_update_staging_duration(&dao, new_staging_duration)
        .await?;
    assert_success_with_unit_return(res.clone());

    // Let some time pass but not enough.
    fast_forward_beyond(&setup.worker, sdk_duration_from_secs(1)).await;

    // Verify trying to apply the new duration fails.
    let res = setup
        .upgradable_contract
        .up_apply_update_staging_duration(&dao)
        .await?;
    assert_failure_with(res, ERR_MSG_UPDATE_DURATION_TOO_EARLY);

    Ok(())
}

/// An account that has been granted an access control role `r` may not successfully call a method
/// that whitelists only roles other than `r`.
#[tokio::test]
async fn test_acl_permission_scope() -> anyhow::Result<()> {
    let worker = near_workspaces::sandbox().await?;
    let setup = Setup::new(worker.clone(), None, None).await?;

    // Create an account and grant it `Role::CodeStager`.
    let code_stager = worker.dev_create_account().await?;
    let granted = setup
        .acl_contract
        .acl_grant_role(setup.contract.as_account(), "CodeStager", code_stager.id())
        .await?;
    assert_eq!(Some(true), granted);

    // Stage some code. Account `code_stager` is authorized to do this.
    let code = vec![1, 2, 3];
    let res = setup
        .upgradable_contract
        .up_stage_code(&code_stager, code.clone())
        .await?;
    assert_success_with_unit_return(res);
    setup.assert_staged_code(Some(code)).await;

    // Verify `code_stager` is not authorized to deploy staged code. Only grantees of at least one
    // of the roles passed as `code_deployers` to the `Upgradable` derive macro are authorized to
    // deploy code.
    let res = setup
        .upgradable_contract
        .up_deploy_code(&setup.unauth_account, None)
        .await?;
    assert_insufficient_acl_permissions(
        res,
        "up_deploy_code",
        &["CodeDeployer".to_string(), "DAO".to_string()],
    );

    // Verify `code` wasn't deployed by calling a function that is defined only in the initial
    // contract but not in the contract corresponding to `code`.
    setup.assert_is_set_up(&setup.unauth_account).await;

    Ok(())
}
