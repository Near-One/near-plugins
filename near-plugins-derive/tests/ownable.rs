// Using `pub` to avoid invalid `dead_code` warnings, see
// https://users.rust-lang.org/t/invalid-dead-code-warning-for-submodule-in-integration-test/80259
pub mod common;

use anyhow::Ok;
use common::key::{delete_access_key, get_access_key_infos};
use common::ownable_contract::OwnableContract;
use common::utils::{
    assert_access_key_not_found_error, assert_only_owner_permission_failure,
    assert_ownable_permission_failure, assert_owner_update_failure, assert_success_with,
};
use near_sdk::serde_json::json;
use near_workspaces::network::Sandbox;
use near_workspaces::result::ExecutionFinalResult;
use near_workspaces::{Account, AccountId, Contract, Worker};
use std::path::Path;

const PROJECT_PATH: &str = "./tests/contracts/ownable";

/// Allows spinning up a setup for testing the contract in [`PROJECT_PATH`] and bundles related
/// resources.
struct Setup {
    /// Instance of the deployed contract.
    contract: Contract,
    /// Wrapper around the deployed contract that facilitates interacting with methods provided by
    /// the `Ownable` plugin.
    ownable_contract: OwnableContract,
    /// A newly created account without any `Ownable` permissions.
    unauth_account: Account,
}

impl Setup {
    /// Deploys and initializes the contract in [`PROJECT_PATH`] and returns a new `Setup`.
    ///
    /// The `owner` parameter is passed on to the contract's constructor, allowing to optionally set
    /// the owner during initialization.
    async fn new(worker: Worker<Sandbox>, owner: Option<AccountId>) -> anyhow::Result<Self> {
        // Compile and deploy the contract.
        let wasm = common::repo::compile_project(Path::new(PROJECT_PATH), "ownable").await?;
        let contract = worker.dev_deploy(&wasm).await?;
        let ownable_contract = OwnableContract::new(contract.clone());

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
            ownable_contract,
            unauth_account,
        })
    }

    /// Calls the contract's `get_counter` method from an account without any `Ownable` permissions.
    async fn get_counter(&self) -> anyhow::Result<u64> {
        let res = self
            .unauth_account
            .call(self.contract.id(), "get_counter")
            .view()
            .await?;
        Ok(res.json::<u64>()?)
    }

    /// Calls one of the methods that increases the counter with signature:
    ///
    /// ```ignore
    /// method_name(&mut self) -> u64
    /// ```
    async fn call_counter_increaser(
        &self,
        caller: &Account,
        method_name: &str,
    ) -> near_workspaces::Result<ExecutionFinalResult> {
        caller
            .call(self.contract.id(), method_name)
            .max_gas()
            .transact()
            .await
    }

    /// Asserts the contract's `owner_get` method returns the expected value.
    async fn assert_owner_is(&self, expected: Option<&AccountId>) {
        // Call from an account without any permissions since `owner_get` is unrestricted.
        let owner = self
            .ownable_contract
            .owner_get(&self.unauth_account)
            .await
            .unwrap();
        assert_eq!(owner.as_ref(), expected);
    }
}

/// Smoke test of contract setup and basic functionality.
#[tokio::test]
async fn test_setup() -> anyhow::Result<()> {
    let worker = near_workspaces::sandbox().await?;
    let setup = Setup::new(worker, None).await?;

    assert_eq!(setup.get_counter().await?, 0);
    let res = setup
        .call_counter_increaser(&setup.unauth_account, "increase")
        .await?;
    assert_success_with(res, 1);
    assert_eq!(setup.get_counter().await?, 1);

    Ok(())
}

#[tokio::test]
async fn test_owner_is() -> anyhow::Result<()> {
    let worker = near_workspaces::sandbox().await?;
    let owner = worker.dev_create_account().await?;
    let setup = Setup::new(worker, Some(owner.id().clone())).await?;

    // Returns false for an account that isn't owner.
    assert!(
        !setup
            .ownable_contract
            .owner_is(&setup.unauth_account)
            .await?
    );

    // Returns true for the owner.
    assert!(setup.ownable_contract.owner_is(&owner).await?);

    Ok(())
}

#[tokio::test]
async fn test_set_owner_ok() -> anyhow::Result<()> {
    let worker = near_workspaces::sandbox().await?;
    let setup = Setup::new(worker, None).await?;

    setup.assert_owner_is(None).await;

    let owner_id = setup.unauth_account.id();
    setup
        .ownable_contract
        .owner_set(setup.contract.as_account(), Some(owner_id.clone()))
        .await?
        .into_result()?;
    setup.assert_owner_is(Some(owner_id)).await;

    Ok(())
}

#[tokio::test]
async fn test_set_owner_fail() -> anyhow::Result<()> {
    let worker = near_workspaces::sandbox().await?;
    let owner = worker.dev_create_account().await?;
    let setup = Setup::new(worker, Some(owner.id().clone())).await?;

    setup.assert_owner_is(Some(owner.id())).await;
    let res = setup
        .ownable_contract
        .owner_set(
            &setup.unauth_account,
            Some(setup.unauth_account.id().clone()),
        )
        .await?;
    assert_owner_update_failure(res);

    Ok(())
}

#[tokio::test]
async fn test_remove_owner() -> anyhow::Result<()> {
    let worker = near_workspaces::sandbox().await?;
    let owner = worker.dev_create_account().await?;
    let setup = Setup::new(worker, Some(owner.id().clone())).await?;

    setup.assert_owner_is(Some(owner.id())).await;

    setup
        .ownable_contract
        .owner_set(&owner, None)
        .await?
        .into_result()?;
    setup.assert_owner_is(None).await;

    Ok(())
}

/// Contract itself may successfully call a method protected by `#[only(self)]`.
#[tokio::test]
async fn test_only_self_ok() -> anyhow::Result<()> {
    let worker = near_workspaces::sandbox().await?;
    let setup = Setup::new(worker, None).await?;

    assert_eq!(setup.get_counter().await?, 0);
    let res = setup
        .call_counter_increaser(setup.contract.as_account(), "increase_4")
        .await?;
    assert_success_with(res, 4);
    assert_eq!(setup.get_counter().await?, 4);

    Ok(())
}

/// A method protected by `#[only(self)]` fails if called from another account.
#[tokio::test]
async fn test_only_self_fail_unauth() -> anyhow::Result<()> {
    let worker = near_workspaces::sandbox().await?;
    let setup = Setup::new(worker, None).await?;

    let res = setup
        .call_counter_increaser(&setup.unauth_account, "increase_4")
        .await?;
    assert_ownable_permission_failure(res);

    Ok(())
}

/// A method protected by `#[only(self)]` fails if called by the owner.
#[tokio::test]
async fn test_only_self_fail_owner() -> anyhow::Result<()> {
    let worker = near_workspaces::sandbox().await?;
    let owner = worker.dev_create_account().await?;
    let setup = Setup::new(worker, Some(owner.id().clone())).await?;

    let res = setup.call_counter_increaser(&owner, "increase_4").await?;
    assert_ownable_permission_failure(res);

    Ok(())
}

/// Calling a method protected by `#[only(owner)]` from the owner succeeds.
#[tokio::test]
async fn test_only_owner_ok() -> anyhow::Result<()> {
    let worker = near_workspaces::sandbox().await?;
    let owner = worker.dev_create_account().await?;
    let setup = Setup::new(worker, Some(owner.id().clone())).await?;

    assert_eq!(setup.get_counter().await?, 0);
    let res = setup.call_counter_increaser(&owner, "increase_3").await?;
    assert_success_with(res, 3);
    assert_eq!(setup.get_counter().await?, 3);

    Ok(())
}

/// A method protected by `#[only(owner)]` fails if called by the contract itself.
#[tokio::test]
async fn test_only_owner_fail_self() -> anyhow::Result<()> {
    let worker = near_workspaces::sandbox().await?;
    let setup = Setup::new(worker, None).await?;

    let res = setup
        .call_counter_increaser(setup.contract.as_account(), "increase_3")
        .await?;
    assert_only_owner_permission_failure(res);

    Ok(())
}

/// A method protected by `#[only(owner)]` fails if called by another account.
#[tokio::test]
async fn test_only_owner_fail() -> anyhow::Result<()> {
    let worker = near_workspaces::sandbox().await?;
    let setup = Setup::new(worker, None).await?;

    let res = setup
        .call_counter_increaser(&setup.unauth_account, "increase_3")
        .await?;
    assert_only_owner_permission_failure(res);

    Ok(())
}

/// Calling a method protected by `#[only(self, owner)]` succeeds if called by the contract itself
/// or by the owner.
#[tokio::test]
async fn test_only_self_owner_ok() -> anyhow::Result<()> {
    let worker = near_workspaces::sandbox().await?;
    let owner = worker.dev_create_account().await?;
    let setup = Setup::new(worker, Some(owner.id().clone())).await?;

    assert_eq!(setup.get_counter().await?, 0);
    let res = setup
        .call_counter_increaser(setup.contract.as_account(), "increase_2")
        .await?;
    assert_success_with(res, 2);
    assert_eq!(setup.get_counter().await?, 2);

    let res = setup.call_counter_increaser(&owner, "increase_2").await?;
    assert_success_with(res, 4);
    assert_eq!(setup.get_counter().await?, 4);

    Ok(())
}

/// Calling a method protected by `#[only(self, owner)]` fails if called by another account.
#[tokio::test]
async fn test_only_self_owner_fail() -> anyhow::Result<()> {
    let worker = near_workspaces::sandbox().await?;
    let setup = Setup::new(worker, None).await?;

    let res = setup
        .call_counter_increaser(&setup.unauth_account, "increase_2")
        .await?;
    assert_ownable_permission_failure(res);

    Ok(())
}

/// Verifies that the contract cannot set a new owner after its access keys are removed.
#[tokio::test]
async fn test_removing_contract_keys_freezes_owner() -> anyhow::Result<()> {
    let worker = near_workspaces::sandbox().await?;
    let owner = worker.dev_create_account().await?;
    let setup = Setup::new(worker, Some(owner.id().clone())).await?;

    setup.assert_owner_is(Some(owner.id())).await;

    // Remove the contract's access key.
    let contract_key = setup.contract.as_account().secret_key().public_key();
    delete_access_key(
        setup.contract.as_account(),
        setup.contract.id(),
        contract_key,
    )
    .await?
    .into_result()?;

    // Assert the contract has no access keys anymore.
    let access_key_infos = get_access_key_infos(&setup.contract).await;
    assert_eq!(access_key_infos.len(), 0, "There should be no access keys");

    // Remove the current owner.
    setup
        .ownable_contract
        .owner_set(&owner, None)
        .await?
        .into_result()?;
    setup.assert_owner_is(None).await;

    // Verify setting a new owner fails since the contract has no access keys.
    let res = setup
        .ownable_contract
        .owner_set(
            setup.contract.as_account(),
            Some(setup.unauth_account.id().clone()),
        )
        .await;
    assert_access_key_not_found_error(res);
    setup.assert_owner_is(None).await;

    Ok(())
}
