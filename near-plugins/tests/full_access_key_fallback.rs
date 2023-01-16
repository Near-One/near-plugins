// Using `pub` to avoid invalid `dead_code` warnings, see
// https://users.rust-lang.org/t/invalid-dead-code-warning-for-submodule-in-integration-test/80259
pub mod common;

use anyhow::Ok;
use common::full_access_key_fallback_contract::FullAccessKeyFallbackContract;
use common::utils::{assert_only_owner_permission_failure, assert_success_with_unit_return};
use near_sdk::serde::Deserialize;
use near_sdk::serde_json::{from_value, json};
use std::iter;
use std::path::Path;
use workspaces::network::Sandbox;
use workspaces::types::{AccessKeyPermission, PublicKey};
use workspaces::{Account, AccountId, Contract, Worker};

const PROJECT_PATH: &str = "./tests/contracts/full_access_key_fallback";

/// Returns a new PublicKey that can be used in tests.
///
/// It returns a `near_sdk::PublicKey` since that's the type required for
/// `FullAccessKeyFallback::attach_full_access_key`.
fn new_public_key() -> near_sdk::PublicKey {
    "ed25519:6E8sCci9badyRkXb3JoRpBj5p8C6Tw41ELDZoiihKEtp"
        .parse()
        .unwrap()
}

/// Converts a `near_sdk::PublicKey` to a `workspaces::types::PublicKey`.
fn pk_sdk_to_workspaces(public_key: near_sdk::PublicKey) -> PublicKey {
    #[derive(Deserialize)]
    struct Wrapper {
        public_key: PublicKey,
    }

    let ser = json!({ "public_key": public_key });
    from_value::<Wrapper>(ser).unwrap().public_key
}

/// Allows spinning up a setup for testing the contract in [`PROJECT_PATH`] and bundles related
/// resources.
struct Setup {
    /// Instance of the deployed contract.
    contract: Contract,
    /// Wrapper around the deployed contract that facilitates interacting with methods provided by
    /// the `FullAccessKeyFallback` plugin.
    fa_key_fallback_contract: FullAccessKeyFallbackContract,
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
        let wasm =
            common::repo::compile_project(Path::new(PROJECT_PATH), "full_access_key_fallback")
                .await?;
        let contract = worker.dev_deploy(&wasm).await?;
        let fa_key_fallback_contract = FullAccessKeyFallbackContract::new(contract.clone());

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
            fa_key_fallback_contract,
            unauth_account,
        })
    }

    /// Asserts the contract's access keys are:
    ///
    /// - the contracts own key followed by
    /// - the keys specified in `keys`
    ///
    /// Moreover, it asserts that all access keys have `FullAccess` permission.
    async fn assert_full_access_keys(&self, keys: &[PublicKey]) {
        // Assert the number of keys.
        let access_keys = self
            .contract
            .view_access_keys()
            .await
            .expect("Should view access keys");
        assert_eq!(
            access_keys.len(),
            keys.len() + 1, // + 1 for the contract's key
        );

        // Assert the `access_keys` are the contract's key followed by `keys` (all full access).
        let contract_key = self.contract.as_account().secret_key().public_key();
        let expected_keys = iter::once(&contract_key).chain(keys.iter());
        for (i, expected_key) in expected_keys.into_iter().enumerate() {
            let access_key = &access_keys[i];
            assert_eq!(
                &access_key.public_key, expected_key,
                "Unexpected PublicKey at index {}",
                i
            );
            println!("looking at {:?}", expected_key);
            assert!(
                matches!(
                    access_key.access_key.permission,
                    AccessKeyPermission::FullAccess,
                ),
                "Unexpected permission of access key at index {}: {:?}",
                i,
                access_key.access_key.permission,
            );
        }
    }
}

/// Smoke test of contract setup.
#[tokio::test]
async fn test_setup() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let _ = Setup::new(worker, None).await?;

    Ok(())
}

#[tokio::test]
async fn test_non_owner_cannot_attach_full_access_key() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let owner = worker.dev_create_account().await?;
    let setup = Setup::new(worker, Some(owner.id().clone())).await?;

    let new_fak = new_public_key();
    let res = setup
        .fa_key_fallback_contract
        .attach_full_access_key(&setup.unauth_account, new_fak)
        .await?;
    assert_only_owner_permission_failure(res);

    Ok(())
}

#[tokio::test]
async fn test_attach_full_access_key() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let owner = worker.dev_create_account().await?;
    let setup = Setup::new(worker, Some(owner.id().clone())).await?;

    // Initially there's just the contract's access key.
    setup.assert_full_access_keys(&[]).await;

    // Owner may attach a full access key.
    let new_fak = new_public_key();
    let res = setup
        .fa_key_fallback_contract
        .attach_full_access_key(&owner, new_fak.clone())
        .await?;
    assert_success_with_unit_return(res);
    setup
        .assert_full_access_keys(&[pk_sdk_to_workspaces(new_fak)])
        .await;

    Ok(())
}
