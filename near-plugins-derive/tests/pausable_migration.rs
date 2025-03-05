pub mod common;

use anyhow::Result;
use common::utils::{
    assert_insufficient_acl_permissions, assert_success_with, assert_success_with_unit_return,
};
use near_sdk::serde_json::json;
use near_workspaces::network::Sandbox;
use near_workspaces::{Account, Contract, Worker};
use std::path::Path;

// Paths to the contract directories
const OLD_CONTRACT_PATH: &str = "./tests/contracts/pausable_old";
const NEW_CONTRACT_PATH: &str = "./tests/contracts/pausable_new";

/// Test struct to manage resources and helper methods
struct MigrationTest {
    worker: Worker<Sandbox>,
    contract: Contract,
    pause_manager: Account,
    unpause_manager: Account,
}

impl MigrationTest {
    /// Deploy the old contract and create test accounts
    async fn new() -> Result<Self> {
        let worker = near_workspaces::sandbox().await?;

        // Compile and deploy the old style contract
        let wasm =
            common::repo::compile_project(Path::new(OLD_CONTRACT_PATH), "pausable_old").await?;
        let contract = worker.dev_deploy(&wasm).await?;

        // Create accounts for testing
        let pause_manager = worker.dev_create_account().await?;
        let unpause_manager = worker.dev_create_account().await?;

        // Initialize the old contract with just the pause_manager
        contract
            .call("new")
            .args_json(json!({
                "pause_manager": pause_manager.id(),
            }))
            .max_gas()
            .transact()
            .await?
            .into_result()?;

        Ok(Self {
            worker,
            contract,
            pause_manager,
            unpause_manager,
        })
    }

    /// Verify the manager has the expected role
    async fn verify_role(&self, account: &Account, role: &str, expected: bool) -> Result<()> {
        let has_role: bool = self
            .contract
            .call("has_role")
            .args_json(json!({
                "role": role,
                "account_id": account.id(),
            }))
            .view()
            .await?
            .json()?;

        assert_eq!(has_role, expected, "Role verification failed for {}", role);
        Ok(())
    }

    /// Pause a feature
    async fn pause_feature(
        &self,
        account: &Account,
        feature: &str,
    ) -> Result<near_workspaces::result::ExecutionFinalResult> {
        let result = account
            .call(self.contract.id(), "pa_pause_feature")
            .args_json(json!({ "key": feature }))
            .max_gas()
            .transact()
            .await?;

        Ok(result)
    }

    /// Unpause a feature
    async fn unpause_feature(
        &self,
        account: &Account,
        feature: &str,
    ) -> Result<near_workspaces::result::ExecutionFinalResult> {
        let result = account
            .call(self.contract.id(), "pa_unpause_feature")
            .args_json(json!({ "key": feature }))
            .max_gas()
            .transact()
            .await?;

        Ok(result)
    }

    /// Check if a feature is paused
    async fn is_paused(&self, account: &Account, feature: &str) -> Result<bool> {
        let result = account
            .call(self.contract.id(), "pa_is_paused")
            .args_json(json!({ "key": feature }))
            .view()
            .await?;

        Ok(result.json()?)
    }

    /// Deploy the new contract code and migrate
    async fn upgrade_contract(&self) -> Result<()> {
        // Compile the new style contract
        let wasm =
            common::repo::compile_project(Path::new(NEW_CONTRACT_PATH), "pausable_new").await?;

        // Deploy the new contract code
        self.contract.as_account().deploy(&wasm).await?.result;

        // Call the migration function to maintain backward compatibility
        let res = self
            .contract
            .call("migrate_pause_unpause_roles")
            .max_gas()
            .transact()
            .await?;

        assert_success_with_unit_return(res);

        // Grant UnpauseManager role to the unpause_manager account
        let res = self
            .contract
            .as_account()
            .call(self.contract.id(), "acl_grant_role")
            .args_json(json!({
                "role": "UnpauseManager",
                "account_id": self.unpause_manager.id(),
            }))
            .max_gas()
            .transact()
            .await?;

        assert_success_with(res, true);

        Ok(())
    }
}

/// Test the migration from old-style pausable to new-style pausable
#[tokio::test]
async fn test_pausable_migration() -> Result<()> {
    // Setup the test with old contract
    let test = MigrationTest::new().await?;

    // Verify initial roles
    test.verify_role(&test.pause_manager, "PauseManager", true)
        .await?;

    // Test that pause_manager can both pause and unpause features in the old contract
    let res = test.pause_feature(&test.pause_manager, "increment").await?;
    assert_success_with(res, true);

    let res = test
        .unpause_feature(&test.pause_manager, "increment")
        .await?;
    assert_success_with(res, true);

    // Upgrade to the new contract
    test.upgrade_contract().await?;

    // After migration, pause_manager should have both roles
    test.verify_role(&test.pause_manager, "PauseManager", true)
        .await?;
    test.verify_role(&test.pause_manager, "UnpauseManager", true)
        .await?;
    test.verify_role(&test.unpause_manager, "UnpauseManager", true)
        .await?;
    test.verify_role(&test.unpause_manager, "PauseManager", false)
        .await?;

    // Test that pause_manager can still pause features
    let res = test.pause_feature(&test.pause_manager, "increment").await?;
    assert_success_with(res, true);

    // Test that pause_manager can still unpause features (due to migration granting both roles)
    let res = test
        .unpause_feature(&test.pause_manager, "increment")
        .await?;
    assert_success_with(res, true);

    // Pause the feature again before testing unpause_manager
    let res = test.pause_feature(&test.pause_manager, "increment").await?;
    assert_success_with(res, true);

    // Verify the feature is actually paused
    let is_paused = test.is_paused(&test.pause_manager, "increment").await?;
    assert!(
        is_paused,
        "Feature should be paused before testing unpause_manager"
    );

    // Test that unpause_manager can unpause features but not pause them
    let res = test
        .pause_feature(&test.unpause_manager, "increment")
        .await?;
    assert_insufficient_acl_permissions(res, "pa_pause_feature", vec!["PauseManager".to_string()]);

    let res = test
        .unpause_feature(&test.unpause_manager, "increment")
        .await?;
    assert_success_with(res, true);

    // Verify the feature was successfully unpaused
    let is_paused = test.is_paused(&test.pause_manager, "increment").await?;
    assert!(
        !is_paused,
        "Feature should be unpaused after unpause_manager action"
    );

    // Add a second account with only PauseManager role
    let pause_only = test.worker.dev_create_account().await?;
    let res = test
        .contract
        .as_account()
        .call(test.contract.id(), "acl_grant_role")
        .args_json(json!({
            "role": "PauseManager",
            "account_id": pause_only.id(),
        }))
        .max_gas()
        .transact()
        .await?;
    assert_success_with(res, true);

    // Verify the new account can pause but not unpause
    let res = test.pause_feature(&pause_only, "increment").await?;
    assert_success_with(res, true);

    let res = test.unpause_feature(&pause_only, "increment").await?;
    assert_insufficient_acl_permissions(
        res,
        "pa_unpause_feature",
        vec!["UnpauseManager".to_string()],
    );

    Ok(())
}
