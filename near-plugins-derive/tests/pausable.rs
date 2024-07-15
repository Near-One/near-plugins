// Using `pub` to avoid invalid `dead_code` warnings, see
// https://users.rust-lang.org/t/invalid-dead-code-warning-for-submodule-in-integration-test/80259
pub mod common;

use common::access_controllable_contract::AccessControllableContract;
use common::pausable_contract::PausableContract;
use common::utils::{
    assert_failure_with, assert_insufficient_acl_permissions, assert_method_is_paused,
    assert_pausable_escape_hatch_is_closed, assert_success_with, assert_success_with_unit_return,
};
use near_sdk::serde_json::json;
use near_workspaces::network::Sandbox;
use near_workspaces::result::ExecutionFinalResult;
use near_workspaces::{Account, AccountId, Contract, Worker};
use std::collections::HashSet;
use std::path::Path;

const PROJECT_PATH: &str = "./tests/contracts/pausable";

/// Bundles resources required in tests.
struct Setup {
    /// The worker interacting with the current sandbox.
    worker: Worker<Sandbox>,
    // Instance of the deployed contract.
    contract: Contract,
    /// Wrapper around the deployed contract that facilitates interacting with
    /// methods provided by the `Pausable` plugin.
    pausable_contract: PausableContract,
    /// Wrapper around the deployed contract that facilitates interacting with
    /// methods provided by the `AccessControllable` plugin.
    acl_contract: AccessControllableContract,
    /// An account with permission to pause and unpause features.
    pause_manager: Account,
    /// A newly created account without any `AccessControllable` permissions.
    unauth_account: Account,
}

impl Setup {
    /// Deploys the contract in [`PROJECT_PATH`] and initializes `Setup`.
    async fn new() -> anyhow::Result<Self> {
        // Compile and deploy the contract.
        let worker = near_workspaces::sandbox().await?;
        let wasm = common::repo::compile_project(Path::new(PROJECT_PATH), "pausable").await?;
        let contract = worker.dev_deploy(&wasm).await?;
        let pausable_contract = PausableContract::new(contract.clone());
        let acl_contract = AccessControllableContract::new(contract.clone());

        // Call the contract's constructor.
        let pause_manager = worker.dev_create_account().await?;
        contract
            .call("new")
            .args_json(json!({
                "pause_manager": pause_manager.id(),
            }))
            .max_gas()
            .transact()
            .await?
            .into_result()?;

        let unauth_account = worker.dev_create_account().await?;
        Ok(Self {
            worker,
            contract,
            pausable_contract,
            acl_contract,
            pause_manager,
            unauth_account,
        })
    }

    /// Grants `role` to `account_id`. Panics if the role is not successfully granted.
    async fn must_grant_acl_role(&self, role: &str, account_id: &AccountId) {
        // The contract itself is made super admin in the constructor, hence this should succeed.
        let result = self
            .acl_contract
            .acl_grant_role(self.contract.as_account(), role, account_id)
            .await
            .unwrap();
        assert_eq!(result, Some(true));
    }

    /// Calls `get_counter` from an account without acl permissions. This method isn't restricted by
    /// acl and cannot be paused.
    async fn get_counter(&self) -> anyhow::Result<u64> {
        let res = self
            .unauth_account
            .call(self.pausable_contract.contract().id(), "get_counter")
            .view()
            .await?;
        Ok(res.json::<u64>()?)
    }

    /// Calls one of the methods that increases or decreases the counter with signature:
    ///
    /// ```ignore
    /// method_name(&mut self) -> ()
    /// ```
    async fn call_counter_modifier(
        &self,
        caller: &Account,
        method_name: &str,
    ) -> near_workspaces::Result<ExecutionFinalResult> {
        caller
            .call(self.pausable_contract.contract().id(), method_name)
            .max_gas()
            .transact()
            .await
    }
}

/// Smoke test of contract setup and basic functionality.
#[tokio::test]
async fn test_setup() -> anyhow::Result<()> {
    let setup = Setup::new().await?;

    setup
        .unauth_account
        .call(setup.pausable_contract.contract().id(), "increase_1")
        .args_json(json!({}))
        .max_gas()
        .transact()
        .await?
        .into_result()?;
    assert_eq!(setup.get_counter().await?, 1);

    Ok(())
}

#[tokio::test]
async fn test_pause_feature() -> anyhow::Result<()> {
    let setup = Setup::new().await?;

    // Pause a feature that is not yet paused.
    let res = setup
        .pausable_contract
        .pa_pause_feature(&setup.pause_manager, "increase_1")
        .await?;
    assert_success_with(res, true);
    let res = setup
        .call_counter_modifier(&setup.unauth_account, "increase_1")
        .await?;
    assert_method_is_paused(res);

    // Pause a feature that is already paused.
    let res = setup
        .pausable_contract
        .pa_pause_feature(&setup.pause_manager, "increase_1")
        .await?;
    assert_success_with(res, false);
    let res = setup
        .call_counter_modifier(&setup.unauth_account, "increase_1")
        .await?;
    assert_method_is_paused(res);

    Ok(())
}

/// A paused method cannot be called from an account with a manager role.
#[tokio::test]
async fn test_pause_feature_from_pause_manager() -> anyhow::Result<()> {
    let setup = Setup::new().await?;
    let res = setup
        .pausable_contract
        .pa_pause_feature(&setup.pause_manager, "increase_1")
        .await?;
    assert_success_with(res, true);
    let res = setup
        .call_counter_modifier(&setup.pause_manager, "increase_1")
        .await?;
    assert_method_is_paused(res);
    Ok(())
}

/// Asserts `pa_pause_feature` fails due to insufficient acl permissions when called by `caller`.
async fn assert_pause_feature_acl_failure(contract: &PausableContract, caller: &Account) {
    let result = contract
        .pa_pause_feature(caller, "increase_1")
        .await
        .unwrap();
    assert_insufficient_acl_permissions(result, "pa_pause_feature", &["PauseManager".to_string()]);
}

#[tokio::test]
/// Only accounts that were granted a manager role may pause features.
async fn test_pause_not_allowed_from_unauthorized_account() -> anyhow::Result<()> {
    let Setup {
        pausable_contract,
        unauth_account,
        ..
    } = Setup::new().await?;
    assert_pause_feature_acl_failure(&pausable_contract, &unauth_account).await;
    Ok(())
}

#[tokio::test]
/// If not granted a manager role, the contract itself may not pause features.
async fn test_pause_not_allowed_from_self() -> anyhow::Result<()> {
    let Setup {
        contract,
        pausable_contract,
        ..
    } = Setup::new().await?;
    assert_pause_feature_acl_failure(&pausable_contract, contract.as_account()).await;
    Ok(())
}

#[tokio::test]
async fn test_unpause_feature() -> anyhow::Result<()> {
    let setup = Setup::new().await?;

    // Pause a feature.
    let res = setup
        .pausable_contract
        .pa_pause_feature(&setup.pause_manager, "increase_1")
        .await?;
    assert_success_with(res, true);
    let res = setup
        .call_counter_modifier(&setup.unauth_account, "increase_1")
        .await?;
    assert_method_is_paused(res);

    // Unpause a feature that is paused. The method it protected can then be called successfully.
    let res = setup
        .pausable_contract
        .pa_unpause_feature(&setup.pause_manager, "increase_1")
        .await?;
    assert_success_with(res, true);
    setup
        .call_counter_modifier(&setup.unauth_account, "increase_1")
        .await?
        .into_result()?;

    // Unpause a feature that is not paused.
    let res = setup
        .pausable_contract
        .pa_unpause_feature(&setup.pause_manager, "increase_1")
        .await?;
    assert_success_with(res, false);
    setup
        .call_counter_modifier(&setup.unauth_account, "increase_1")
        .await?
        .into_result()?;

    Ok(())
}

/// Asserts `pa_unpause_feature` fails due to insufficient acl permissions when called by `caller`.
async fn assert_unpause_feature_acl_failure(contract: &PausableContract, caller: &Account) {
    let result = contract
        .pa_unpause_feature(caller, "increase_1")
        .await
        .unwrap();
    assert_insufficient_acl_permissions(
        result,
        "pa_unpause_feature",
        &["PauseManager".to_string()],
    );
}

#[tokio::test]
/// Only accounts that were granted a manager role may unpause features.
async fn test_unpause_not_allowed_from_unauthorized_account() -> anyhow::Result<()> {
    let Setup {
        pausable_contract,
        unauth_account,
        ..
    } = Setup::new().await?;
    assert_unpause_feature_acl_failure(&pausable_contract, &unauth_account).await;
    Ok(())
}

#[tokio::test]
/// If not granted a manager role, the contract itself may not unpause features.
async fn test_unpause_not_allowed_from_self() -> anyhow::Result<()> {
    let Setup {
        contract,
        pausable_contract,
        ..
    } = Setup::new().await?;
    assert_unpause_feature_acl_failure(&pausable_contract, contract.as_account()).await;
    Ok(())
}

#[tokio::test]
async fn test_pause_with_all() -> anyhow::Result<()> {
    let setup = Setup::new().await?;
    let res = setup
        .pausable_contract
        .pa_pause_feature(&setup.pause_manager, "ALL")
        .await?;
    assert_success_with(res, true);
    let res = setup
        .call_counter_modifier(&setup.unauth_account, "increase_1")
        .await?;
    assert_method_is_paused(res);
    Ok(())
}

/// Verify `except` escape hatch works when the feature is paused via `ALL`.
#[tokio::test]
async fn test_pause_with_all_allows_except() -> anyhow::Result<()> {
    let setup = Setup::new().await?;
    let res = setup
        .pausable_contract
        .pa_pause_feature(&setup.pause_manager, "ALL")
        .await?;
    assert_success_with(res, true);

    let exempted_account = setup.unauth_account.clone();
    setup
        .must_grant_acl_role("Unrestricted4Increaser", exempted_account.id())
        .await;

    let res = setup
        .call_counter_modifier(&exempted_account, "increase_4")
        .await?;
    assert_success_with_unit_return(res);
    assert_eq!(setup.get_counter().await?, 4);
    Ok(())
}

#[tokio::test]
async fn test_not_paused_with_different_key() -> anyhow::Result<()> {
    let setup = Setup::new().await?;
    let res = setup
        .pausable_contract
        .pa_pause_feature(&setup.pause_manager, "other_feature")
        .await?;
    assert_success_with(res, true);

    let res = setup
        .call_counter_modifier(&setup.unauth_account, "increase_1")
        .await?;
    assert_success_with_unit_return(res);
    assert_eq!(setup.get_counter().await?, 1);

    Ok(())
}

#[tokio::test]
async fn test_work_after_unpause() -> anyhow::Result<()> {
    let setup = Setup::new().await?;

    // After pausing function call fails.
    let res = setup
        .pausable_contract
        .pa_pause_feature(&setup.pause_manager, "increase_1")
        .await?;
    assert_success_with(res, true);
    let res = setup
        .call_counter_modifier(&setup.unauth_account, "increase_1")
        .await?;
    assert_method_is_paused(res);

    // After unpausing function call succeeds.
    let res = setup
        .pausable_contract
        .pa_unpause_feature(&setup.pause_manager, "increase_1")
        .await?;
    assert_success_with(res, true);
    let res = setup
        .call_counter_modifier(&setup.unauth_account, "increase_1")
        .await?;
    assert_success_with_unit_return(res);
    assert_eq!(setup.get_counter().await?, 1);

    Ok(())
}

async fn assert_paused_list(
    expected: Option<HashSet<String>>,
    contract: &PausableContract,
    caller: &Account,
) {
    let paused_list = contract.pa_all_paused(caller).await.unwrap();
    assert_eq!(paused_list, expected);
}

#[tokio::test]
async fn test_paused_list() -> anyhow::Result<()> {
    let setup = Setup::new().await?;

    assert_paused_list(None, &setup.pausable_contract, &setup.unauth_account).await;

    let res = setup
        .pausable_contract
        .pa_pause_feature(&setup.pause_manager, "feature_a")
        .await?;
    assert_success_with(res, true);
    assert_paused_list(
        Some(HashSet::from(["feature_a".to_string()])),
        &setup.pausable_contract,
        &setup.unauth_account,
    )
    .await;

    let res = setup
        .pausable_contract
        .pa_pause_feature(&setup.pause_manager, "feature_b")
        .await?;
    assert_success_with(res, true);
    assert_paused_list(
        Some(HashSet::from([
            "feature_a".to_string(),
            "feature_b".to_string(),
        ])),
        &setup.pausable_contract,
        &setup.unauth_account,
    )
    .await;

    let res = setup
        .pausable_contract
        .pa_unpause_feature(&setup.pause_manager, "feature_a")
        .await?;
    assert_success_with(res, true);
    assert_paused_list(
        Some(HashSet::from(["feature_b".to_string()])),
        &setup.pausable_contract,
        &setup.unauth_account,
    )
    .await;

    Ok(())
}

async fn assert_is_paused(expected: bool, key: &str, contract: &PausableContract, caller: Account) {
    let is_paused = contract.pa_is_paused(&caller, key).await.unwrap();
    assert_eq!(is_paused, expected);
}

#[tokio::test]
async fn test_is_paused() -> anyhow::Result<()> {
    let setup = Setup::new().await?;

    assert_is_paused(
        false,
        "feature_a",
        &setup.pausable_contract,
        setup.unauth_account.clone(),
    )
    .await;

    let res = setup
        .pausable_contract
        .pa_pause_feature(&setup.pause_manager, "feature_a")
        .await?;
    assert_success_with(res, true);
    assert_is_paused(
        true,
        "feature_a",
        &setup.pausable_contract,
        setup.unauth_account.clone(),
    )
    .await;

    let res = setup
        .pausable_contract
        .pa_unpause_feature(&setup.pause_manager, "feature_a")
        .await?;
    assert_success_with(res, true);
    assert_is_paused(
        false,
        "feature_a",
        &setup.pausable_contract,
        setup.unauth_account.clone(),
    )
    .await;

    Ok(())
}

/// Pausing method name has no effect if the method has a custom feature name.
#[tokio::test]
async fn test_pause_custom_name_ok() -> anyhow::Result<()> {
    let setup = Setup::new().await?;

    let res = setup
        .pausable_contract
        .pa_pause_feature(&setup.pause_manager, "increase_2")
        .await?;
    assert_success_with(res, true);

    let res = setup
        .call_counter_modifier(&setup.unauth_account, "increase_2")
        .await?;
    assert_success_with_unit_return(res);
    assert_eq!(setup.get_counter().await?, 2);

    Ok(())
}

#[tokio::test]
async fn test_pause_custom_name_fail() -> anyhow::Result<()> {
    let setup = Setup::new().await?;

    let res = setup
        .pausable_contract
        .pa_pause_feature(&setup.pause_manager, "Increase by two")
        .await?;
    assert_success_with(res, true);

    let res = setup
        .call_counter_modifier(&setup.unauth_account, "increase_2")
        .await?;
    assert_method_is_paused(res);

    Ok(())
}

#[tokio::test]
async fn test_pause_except_ok() -> anyhow::Result<()> {
    let setup = Setup::new().await?;

    // Pause feature.
    let res = setup
        .pausable_contract
        .pa_pause_feature(&setup.pause_manager, "increase_4")
        .await?;
    assert_success_with(res, true);

    // Grantee of `Role::Unrestricted4Increaser` is exempted.
    let increaser = setup.worker.dev_create_account().await?;
    setup
        .must_grant_acl_role("Unrestricted4Increaser", increaser.id())
        .await;
    let res = setup
        .call_counter_modifier(&increaser, "increase_4")
        .await?;
    assert_success_with_unit_return(res);
    assert_eq!(setup.get_counter().await?, 4);

    // Grantee of `Role::Unrestricted4Modifier` is exempted.
    let modifier = setup.worker.dev_create_account().await?;
    setup
        .must_grant_acl_role("Unrestricted4Modifier", modifier.id())
        .await;
    let res = setup.call_counter_modifier(&modifier, "increase_4").await?;
    assert_success_with_unit_return(res);
    assert_eq!(setup.get_counter().await?, 8);

    Ok(())
}

/// If a paused method exempts grantees of roles via `except`, calling that method from an account
/// without an excepted role fails.
#[tokio::test]
async fn test_pause_except_fail() -> anyhow::Result<()> {
    let setup = Setup::new().await?;

    // Pause feature.
    let res = setup
        .pausable_contract
        .pa_pause_feature(&setup.pause_manager, "increase_4")
        .await?;
    assert_success_with(res, true);

    // Calling the method from an account which is not exempted fails.
    let res = setup
        .call_counter_modifier(&setup.unauth_account, "increase_4")
        .await?;
    assert_method_is_paused(res);

    Ok(())
}

#[tokio::test]
async fn test_custom_big_ok() -> anyhow::Result<()> {
    let setup = Setup::new().await?;
    for _ in 0..5 {
        let res = setup
            .call_counter_modifier(&setup.unauth_account, "careful_increase")
            .await?;
        assert_success_with_unit_return(res);
    }
    assert_eq!(setup.get_counter().await?, 5);
    Ok(())
}

#[tokio::test]
async fn test_custom_big_fail() -> anyhow::Result<()> {
    let setup = Setup::new().await?;

    // Pause feature.
    let res = setup
        .pausable_contract
        .pa_pause_feature(&setup.pause_manager, "increase_big")
        .await?;
    assert_success_with(res, true);

    // Counter can still be increased until threshold.
    for _ in 0..3 {
        let res = setup
            .call_counter_modifier(&setup.unauth_account, "careful_increase")
            .await?;
        assert_success_with_unit_return(res);
    }

    // After the threshold the method fails.
    let res = setup
        .call_counter_modifier(&setup.unauth_account, "careful_increase")
        .await?;
    assert_failure_with(res, "Method paused for large values of counter");

    Ok(())
}

/// Calling the method succeeds if the corresponding feature is paused.
#[tokio::test]
async fn test_escape_hatch_ok() -> anyhow::Result<()> {
    let setup = Setup::new().await?;

    // Make counter decreasable.
    let res = setup
        .call_counter_modifier(&setup.unauth_account, "increase_1")
        .await?;
    assert_success_with_unit_return(res);
    assert_eq!(setup.get_counter().await?, 1);

    // Pause feature.
    let res = setup
        .pausable_contract
        .pa_pause_feature(&setup.pause_manager, "increase_1")
        .await?;
    assert_success_with(res, true);

    // Calling escape hatch succeeds.
    let res = setup
        .call_counter_modifier(&setup.unauth_account, "decrease_1")
        .await?;
    assert_success_with_unit_return(res);
    assert_eq!(setup.get_counter().await?, 0);

    Ok(())
}

/// Calling the method fails if the corresponding feature is not paused.
#[tokio::test]
async fn test_escape_hatch_fail() -> anyhow::Result<()> {
    let setup = Setup::new().await?;
    let res = setup
        .call_counter_modifier(&setup.unauth_account, "decrease_1")
        .await?;
    assert_pausable_escape_hatch_is_closed(res, "increase_1");
    Ok(())
}
