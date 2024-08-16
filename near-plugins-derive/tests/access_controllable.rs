// Using `pub` to avoid invalid `dead_code` warnings, see
// https://users.rust-lang.org/t/invalid-dead-code-warning-for-submodule-in-integration-test/80259
pub mod common;

use common::access_controllable_contract::AccessControllableContract;
use common::utils::{
    as_sdk_account_id, assert_insufficient_acl_permissions, assert_private_method_failure,
    assert_success_with,
};
use near_plugins::access_controllable::{PermissionedAccounts, PermissionedAccountsPerRole};
use near_sdk::serde_json::{self, json};
use near_workspaces::network::Sandbox;
use near_workspaces::result::ExecutionFinalResult;
use near_workspaces::{Account, AccountId, Contract, Worker};
use std::collections::{HashMap, HashSet};
use std::convert::TryFrom;
use std::path::Path;

const PROJECT_PATH: &str = "./tests/contracts/access_controllable";

/// All roles which are defined in the contract in [`PROJECT_PATH`].
const ALL_ROLES: [&str; 3] = ["ByMax2Increaser", "ByMax3Increaser", "Resetter"];

/// Bundles resources required in tests.
struct Setup {
    /// The worker interacting with the current sandbox.
    worker: Worker<Sandbox>,
    /// Deployed instance of the contract in [`PROJECT_PATH`].
    contract: AccessControllableContract,
    /// A newly created account (which differs from the contract).
    account: Account,
}

impl Setup {
    fn contract_account(&self) -> &Account {
        self.contract.contract().as_account()
    }

    /// Deploys the contract and calls the initialization method without passing any accounts to be
    /// added as admin or grantees.
    async fn new() -> anyhow::Result<Self> {
        Self::new_with_admins_and_grantees(HashMap::default(), HashMap::default()).await
    }

    /// Deploys the contract with a specific wasm binary.
    async fn new_with_wasm(wasm: Vec<u8>) -> anyhow::Result<Self> {
        Self::deploy_contract(
            wasm,
            json!({
                "admins": HashMap::<String, AccountId>::new(),
                "grantees": HashMap::<String, AccountId>::new()
            }),
        )
        .await
    }

    /// Deploys the contract and passes `admins` and `grantees` to the initialization method. Note
    /// that accounts corresponding to the ids in `admins` and `grantees` are _not_ created.
    async fn new_with_admins_and_grantees(
        admins: HashMap<String, AccountId>,
        grantees: HashMap<String, AccountId>,
    ) -> anyhow::Result<Self> {
        let wasm =
            common::repo::compile_project(Path::new(PROJECT_PATH), "access_controllable").await?;

        Self::deploy_contract(wasm, json!({ "admins": admins, "grantees": grantees })).await
    }

    async fn deploy_contract(wasm: Vec<u8>, args: serde_json::Value) -> anyhow::Result<Self> {
        let worker = near_workspaces::sandbox().await?;
        let contract = AccessControllableContract::new(worker.dev_deploy(&wasm).await?);
        let account = worker.dev_create_account().await?;

        contract
            .contract()
            .call("new")
            .args_json(args)
            .max_gas()
            .transact()
            .await?
            .into_result()?;

        Ok(Self {
            worker,
            contract,
            account,
        })
    }

    /// Returns a new account that is super-admin.
    async fn new_super_admin_account(&self) -> anyhow::Result<Account> {
        let account = self.worker.dev_create_account().await?;
        self.contract
            .acl_add_super_admin_unchecked(self.contract_account(), account.id())
            .await?
            .into_result()?;
        Ok(account)
    }

    /// Returns a new account that is admin for `roles`.
    async fn new_account_as_admin(&self, roles: &[&str]) -> anyhow::Result<Account> {
        let account = self.worker.dev_create_account().await?;
        for &role in roles {
            self.contract
                .acl_add_admin_unchecked(self.contract_account(), role, account.id())
                .await?
                .into_result()?;
        }
        Ok(account)
    }

    async fn new_account_with_roles(&self, roles: &[&str]) -> anyhow::Result<Account> {
        let account = self.worker.dev_create_account().await?;
        for &role in roles {
            self.contract
                .acl_grant_role_unchecked(self.contract_account(), role, account.id())
                .await?
                .into_result()?;
        }
        Ok(account)
    }
}

async fn call_increase_2(
    contract: &Contract,
    caller: &Account,
) -> near_workspaces::Result<ExecutionFinalResult> {
    caller
        .call(contract.id(), "increase_2")
        .args_json(())
        .max_gas()
        .transact()
        .await
}

/// Returns new `PermissionedAccounts` for [`ALL_ROLES`].
fn new_permissioned_accounts() -> PermissionedAccounts {
    let mut permissioned_accounts = PermissionedAccounts {
        super_admins: vec![],
        roles: HashMap::new(),
    };

    for role in ALL_ROLES {
        permissioned_accounts.roles.insert(
            role.to_string(),
            PermissionedAccountsPerRole {
                admins: vec![],
                grantees: vec![],
            },
        );
    }

    permissioned_accounts
}

/// Asserts both `PermissionedAcccounts` contain the same accounts with the same permissions,
/// disregarding order.
///
/// Expects both `a` and `b` to contain every role in [`ALL_ROLES`].
///
/// This function is available only in tests and used for small numbers of accounts, so simplicity
/// is favored over efficiency.
fn assert_permissioned_account_equivalence(a: &PermissionedAccounts, b: &PermissionedAccounts) {
    // Verify super admins.
    assert_account_ids_equivalence(
        a.super_admins.as_slice(),
        b.super_admins.as_slice(),
        "super_admins",
    );

    // Verify admins and grantees per role.
    assert_eq!(a.roles.len(), b.roles.len(), "Unequal number of roles");
    assert_eq!(a.roles.len(), ALL_ROLES.len(), "More roles than expected");
    for role in ALL_ROLES {
        let per_role_a = a
            .roles
            .get(role)
            .unwrap_or_else(|| panic!("PermissionedAccounts a misses role {role}"));
        let per_role_b = b
            .roles
            .get(role)
            .unwrap_or_else(|| panic!("PermissionedAccounts b misses role {role}"));

        assert_account_ids_equivalence(
            &per_role_a.admins,
            &per_role_b.admins,
            &format!("admins of role {role}"),
        );
        assert_account_ids_equivalence(
            &per_role_a.grantees,
            &per_role_b.grantees,
            &format!("grantees of role {role}"),
        );
    }
}

/// Asserts `a` and `b` contain the same `AccountId`s, disregarding order. Parameter `specifier` is
/// passed to the panic message in case of a mismatch.
///
/// This function is available only in tests and used for small numbers of accounts, so simplicity
/// is favored over efficiency.
fn assert_account_ids_equivalence(
    a: &[near_sdk::AccountId],
    b: &[near_sdk::AccountId],
    specifier: &str,
) {
    let set_a: HashSet<_> = a.iter().cloned().collect();
    let set_b: HashSet<_> = b.iter().cloned().collect();
    assert_eq!(set_a, set_b, "Unequal sets of AccountIds for {specifier}");
}

/// Smoke test of contract setup and basic functionality.
#[tokio::test]
async fn test_increase_and_get_counter() -> anyhow::Result<()> {
    let Setup {
        contract, account, ..
    } = Setup::new().await?;
    let contract = contract.contract();

    account
        .call(contract.id(), "increase")
        .max_gas()
        .transact()
        .await?
        .into_result()?;

    let res: u64 = account
        .call(contract.id(), "get_counter")
        .view()
        .await?
        .json()?;

    assert_eq!(res, 1);
    Ok(())
}

#[tokio::test]
async fn test_acl_initialization_in_constructor() -> anyhow::Result<()> {
    let admin_id: AccountId = "admin.acl_test.near".parse().unwrap();
    let grantee_id: AccountId = "grantee.acl_test.near".parse().unwrap();
    let setup = Setup::new_with_admins_and_grantees(
        HashMap::from([("ByMax2Increaser".to_string(), admin_id.clone())]),
        HashMap::from([("Resetter".to_string(), grantee_id.clone())]),
    )
    .await?;

    setup
        .contract
        .assert_acl_is_admin(true, "ByMax2Increaser", &admin_id)
        .await;
    setup
        .contract
        .assert_acl_has_role(true, "Resetter", &grantee_id)
        .await;

    Ok(())
}

#[tokio::test]
async fn test_acl_role_variants() -> anyhow::Result<()> {
    let setup = Setup::new().await?;
    let variants = setup.contract.acl_role_variants(&setup.account).await?;
    assert_eq!(variants, ALL_ROLES);
    Ok(())
}

#[tokio::test]
async fn test_acl_is_super_admin() -> anyhow::Result<()> {
    let Setup {
        contract, account, ..
    } = Setup::new().await?;

    let is_super_admin = contract.acl_is_super_admin(&account, account.id()).await?;
    assert!(!is_super_admin);

    contract
        .acl_add_super_admin_unchecked(contract.contract().as_account(), account.id())
        .await?
        .into_result()?;

    let is_super_admin = contract.acl_is_super_admin(&account, account.id()).await?;
    assert!(is_super_admin);

    Ok(())
}

#[tokio::test]
async fn test_acl_init_super_admin() -> anyhow::Result<()> {
    let Setup {
        worker,
        contract,
        account,
        ..
    } = Setup::new().await?;
    let contract_account = contract.contract().as_account();

    // Calling `acl_init_super_admin` after initialization adds super-admin.
    contract
        .assert_acl_is_super_admin(false, contract_account, account.id())
        .await;
    let res = contract
        .acl_init_super_admin(contract_account, account.id())
        .await?;
    assert_success_with(res, true);
    contract
        .assert_acl_is_super_admin(true, contract_account, account.id())
        .await;

    // Once there's a super-admin, `acl_init_super_admin` returns `false`.
    let res = contract
        .acl_init_super_admin(contract_account, account.id())
        .await?;
    assert_success_with(res, false);

    let other_account = worker.dev_create_account().await?;
    let res = contract
        .acl_init_super_admin(contract_account, other_account.id())
        .await?;
    assert_success_with(res, false);
    contract
        .assert_acl_is_super_admin(false, contract_account, other_account.id())
        .await;

    // When all super-admins have been removed, it succeeds again.
    let res = contract
        .acl_revoke_super_admin_unchecked(contract_account, account.id())
        .await?;
    assert_success_with(res, true);
    let res = contract
        .acl_init_super_admin(contract_account, other_account.id())
        .await?;
    assert_success_with(res, true);
    contract
        .assert_acl_is_super_admin(true, contract_account, other_account.id())
        .await;

    Ok(())
}

#[tokio::test]
async fn test_acl_add_super_admin() -> anyhow::Result<()> {
    let setup = Setup::new().await?;
    let to_be_super_admin = setup.worker.dev_create_account().await?;

    // Create accounts that add a super-admin.
    let caller_unauth = setup.worker.dev_create_account().await?;
    let caller_auth = setup.new_super_admin_account().await?;

    // Adding is a no-op if the caller is not a super-admin.
    let res = setup
        .contract
        .acl_add_super_admin(&caller_unauth, to_be_super_admin.id())
        .await?;
    assert_eq!(res, None);
    setup
        .contract
        .assert_acl_is_super_admin(false, setup.contract_account(), to_be_super_admin.id())
        .await;

    // Adding succeeds if the caller is a super-admin.
    let res = setup
        .contract
        .acl_add_super_admin(&caller_auth, to_be_super_admin.id())
        .await?;
    assert_eq!(res, Some(true));
    setup
        .contract
        .assert_acl_is_super_admin(true, setup.contract_account(), to_be_super_admin.id())
        .await;

    // Adding an account which is already super-admin returns `Some(false)`.
    let res = setup
        .contract
        .acl_add_super_admin(&caller_auth, to_be_super_admin.id())
        .await?;
    assert_eq!(res, Some(false));

    Ok(())
}

#[tokio::test]
async fn test_acl_add_super_admin_unchecked() -> anyhow::Result<()> {
    let Setup {
        contract, account, ..
    } = Setup::new().await?;
    let contract_account = contract.contract().as_account();

    contract
        .assert_acl_is_super_admin(false, contract_account, account.id())
        .await;
    let res = contract
        .acl_add_super_admin_unchecked(contract_account, account.id())
        .await?;
    assert_success_with(res, true);
    contract
        .assert_acl_is_super_admin(true, contract_account, account.id())
        .await;

    // Adding as super-admin again behaves as expected.
    let res = contract
        .acl_add_super_admin_unchecked(contract_account, account.id())
        .await?;
    assert_success_with(res, false);

    Ok(())
}

#[tokio::test]
async fn test_acl_revoke_super_admin() -> anyhow::Result<()> {
    let setup = Setup::new().await?;
    let super_admin = setup.new_super_admin_account().await?;

    setup
        .contract
        .assert_acl_is_super_admin(true, setup.contract_account(), super_admin.id())
        .await;

    // Create revoker accounts.
    let revoker_unauth = setup.worker.dev_create_account().await?;
    let revoker_auth = setup.new_super_admin_account().await?;

    // Revoke is a no-op if revoker is not a super-admin.
    let res = setup
        .contract
        .acl_revoke_super_admin(&revoker_unauth, super_admin.id())
        .await?;
    assert_eq!(res, None);
    setup
        .contract
        .assert_acl_is_super_admin(true, setup.contract_account(), super_admin.id())
        .await;

    // Revoke succeeds if the revoker is a super-admin.
    let res = setup
        .contract
        .acl_revoke_super_admin(&revoker_auth, super_admin.id())
        .await?;
    assert_eq!(res, Some(true));
    setup
        .contract
        .assert_acl_is_super_admin(false, setup.contract_account(), super_admin.id())
        .await;

    // Revoking from an account which isn't super-admin returns `Some(false)`.
    let account = setup.worker.dev_create_account().await?;
    let res = setup
        .contract
        .acl_revoke_super_admin(&revoker_auth, account.id())
        .await?;
    assert_eq!(res, Some(false));

    Ok(())
}

#[tokio::test]
async fn test_acl_transfer_super_admin() -> anyhow::Result<()> {
    let setup = Setup::new().await?;
    let super_admin = setup.new_super_admin_account().await?;
    let new_super_admin = setup.worker.dev_create_account().await?;

    setup
        .contract
        .assert_acl_is_super_admin(true, setup.contract_account(), super_admin.id())
        .await;

    // Create caller account.
    let caller_unauth = setup.worker.dev_create_account().await?;

    // Transfer is a no-op if caller is not a super-admin.
    let res = setup
        .contract
        .acl_transfer_super_admin(&caller_unauth, super_admin.id())
        .await?;
    assert_eq!(res, None);
    setup
        .contract
        .assert_acl_is_super_admin(true, setup.contract_account(), super_admin.id())
        .await;
    setup
        .contract
        .assert_acl_is_super_admin(false, setup.contract_account(), new_super_admin.id())
        .await;

    // Transfer succeeds if the caller is a super-admin.
    let res = setup
        .contract
        .acl_transfer_super_admin(&super_admin, new_super_admin.id())
        .await?;
    assert_eq!(res, Some(true));
    setup
        .contract
        .assert_acl_is_super_admin(false, setup.contract_account(), super_admin.id())
        .await;
    setup
        .contract
        .assert_acl_is_super_admin(true, setup.contract_account(), new_super_admin.id())
        .await;

    // Transfer to an account that is already super-admin returns `Some(false)`.
    let admin = setup.new_super_admin_account().await?;
    let res = setup
        .contract
        .acl_transfer_super_admin(&new_super_admin, admin.id())
        .await?;
    assert_eq!(res, Some(false));

    Ok(())
}

#[tokio::test]
async fn test_acl_revoke_super_admin_unchecked() -> anyhow::Result<()> {
    let setup = Setup::new().await?;
    let account = setup.new_super_admin_account().await?;

    setup
        .contract
        .assert_acl_is_super_admin(true, setup.contract_account(), account.id())
        .await;

    // Revoke an existing super-admin permission.
    let res = setup
        .contract
        .acl_revoke_super_admin_unchecked(setup.contract_account(), account.id())
        .await?;
    assert_success_with(res, true);
    setup
        .contract
        .assert_acl_is_super_admin(false, setup.contract_account(), account.id())
        .await;

    // Revoke from an account which is not super-admin.
    let res = setup
        .contract
        .acl_revoke_super_admin_unchecked(setup.contract_account(), account.id())
        .await?;
    assert_success_with(res, false);

    Ok(())
}

/// Verify that a super-admin is admin for every role.
#[tokio::test]
async fn test_super_admin_is_any_admin() -> anyhow::Result<()> {
    let setup = Setup::new().await?;
    let super_admin = setup.new_super_admin_account().await?;

    for role in ALL_ROLES {
        setup
            .contract
            .assert_acl_is_admin(true, role, super_admin.id())
            .await;
    }

    Ok(())
}

/// Verify that a super-admin may add admins for every role.
#[tokio::test]
async fn test_super_admin_may_add_any_admin() -> anyhow::Result<()> {
    let setup = Setup::new().await?;
    let super_admin = setup.new_super_admin_account().await?;

    for role in ALL_ROLES {
        let account = setup.worker.dev_create_account().await?;
        setup
            .contract
            .assert_acl_is_admin(false, role, account.id())
            .await;

        let res = setup
            .contract
            .acl_add_admin(&super_admin, role, account.id())
            .await?;
        assert_eq!(res, Some(true));
        setup
            .contract
            .assert_acl_is_admin(true, role, account.id())
            .await;
    }

    Ok(())
}

/// Verify that a super-admin may revoke admins for every role.
#[tokio::test]
async fn test_super_admin_may_revoke_any_admin() -> anyhow::Result<()> {
    let setup = Setup::new().await?;
    let super_admin = setup.new_super_admin_account().await?;

    for role in ALL_ROLES {
        let admin = setup.new_account_as_admin(&[role]).await?;
        setup
            .contract
            .assert_acl_is_admin(true, role, admin.id())
            .await;

        let res = setup
            .contract
            .acl_revoke_admin(&super_admin, role, admin.id())
            .await?;
        assert_eq!(res, Some(true));
        setup
            .contract
            .assert_acl_is_admin(false, role, admin.id())
            .await;
    }

    Ok(())
}

/// Verify that a super-admin may grant every role.
#[tokio::test]
async fn test_super_admin_may_grant_any_role() -> anyhow::Result<()> {
    let setup = Setup::new().await?;
    let super_admin = setup.new_super_admin_account().await?;

    for role in ALL_ROLES {
        let account = setup.worker.dev_create_account().await?;
        setup
            .contract
            .assert_acl_has_role(false, role, account.id())
            .await;

        let res = setup
            .contract
            .acl_grant_role(&super_admin, role, account.id())
            .await?;
        assert_eq!(res, Some(true));
        setup
            .contract
            .assert_acl_has_role(true, role, account.id())
            .await;
    }

    Ok(())
}

/// Verify that a super-admin may revoke every role.
#[tokio::test]
async fn test_super_admin_may_revoke_any_role() -> anyhow::Result<()> {
    let setup = Setup::new().await?;
    let super_admin = setup.new_super_admin_account().await?;

    for role in ALL_ROLES {
        let grantee = setup.new_account_with_roles(&[role]).await?;
        setup
            .contract
            .assert_acl_has_role(true, role, grantee.id())
            .await;

        let res = setup
            .contract
            .acl_revoke_role(&super_admin, role, grantee.id())
            .await?;
        assert_eq!(res, Some(true));
        setup
            .contract
            .assert_acl_has_role(false, role, grantee.id())
            .await;
    }

    Ok(())
}

#[tokio::test]
async fn test_acl_is_admin() -> anyhow::Result<()> {
    let Setup {
        contract, account, ..
    } = Setup::new().await?;
    let contract_account = contract.contract().as_account();
    let role = "ByMax2Increaser";

    let is_admin = contract.acl_is_admin(&account, role, account.id()).await?;
    assert!(!is_admin);

    contract
        .acl_add_admin_unchecked(contract_account, role, account.id())
        .await?
        .into_result()?;

    let is_admin = contract.acl_is_admin(&account, role, account.id()).await?;
    assert!(is_admin);

    Ok(())
}

#[tokio::test]
async fn test_acl_add_admin() -> anyhow::Result<()> {
    let Setup {
        worker,
        contract,
        account,
        ..
    } = Setup::new().await?;
    let contract_account = contract.contract().as_account();
    let role = "ByMax2Increaser";

    let acc_adding_admin = account;
    let acc_to_be_admin = worker.dev_create_account().await?;

    contract
        .assert_acl_is_admin(false, role, acc_to_be_admin.id())
        .await;

    // An account which isn't admin can't add admins.
    let added = contract
        .acl_add_admin(&acc_adding_admin, role, acc_to_be_admin.id())
        .await?;
    assert_eq!(added, None);

    // Admin can add others as admin.
    contract
        .acl_add_admin_unchecked(contract_account, role, acc_adding_admin.id())
        .await?
        .into_result()?;
    let added = contract
        .acl_add_admin(&acc_adding_admin, role, acc_to_be_admin.id())
        .await?;
    assert_eq!(added, Some(true));
    contract
        .assert_acl_is_admin(true, role, acc_to_be_admin.id())
        .await;

    // Adding an account that is already admin.
    let added = contract
        .acl_add_admin(&acc_adding_admin, role, acc_to_be_admin.id())
        .await?;
    assert_eq!(added, Some(false));

    Ok(())
}

#[tokio::test]
async fn test_acl_add_admin_unchecked() -> anyhow::Result<()> {
    let Setup {
        contract, account, ..
    } = Setup::new().await?;
    let contract_account = contract.contract().as_account();
    let role = "ByMax2Increaser";

    contract
        .assert_acl_is_admin(false, role, account.id())
        .await;
    let res = contract
        .acl_add_admin_unchecked(contract_account, role, account.id())
        .await?;
    assert_success_with(res, true);
    contract.assert_acl_is_admin(true, role, account.id()).await;

    // Adding as admin again behaves as expected.
    let res = contract
        .acl_add_admin_unchecked(contract_account, role, account.id())
        .await?;
    assert_success_with(res, false);

    Ok(())
}

#[tokio::test]
async fn test_acl_revoke_admin() -> anyhow::Result<()> {
    let setup = Setup::new().await?;
    let role = "ByMax3Increaser";
    let admin = setup.new_account_as_admin(&[role]).await?;

    setup
        .contract
        .assert_acl_is_admin(true, role, admin.id())
        .await;

    // Revoke is a no-op if revoker is not an admin for the role.
    let revoker = setup.new_account_as_admin(&[]).await?;
    let res = setup
        .contract
        .acl_revoke_admin(&revoker, role, admin.id())
        .await?;
    assert_eq!(res, None);
    let revoker = setup.new_account_as_admin(&["ByMax2Increaser"]).await?;
    let res = setup
        .contract
        .acl_revoke_admin(&revoker, role, admin.id())
        .await?;
    assert_eq!(res, None);
    setup
        .contract
        .assert_acl_is_admin(true, role, admin.id())
        .await;

    // Revoke succeeds if the revoker is an admin for the role.
    let revoker = setup.new_account_as_admin(&[role]).await?;
    let res = setup
        .contract
        .acl_revoke_admin(&revoker, role, admin.id())
        .await?;
    assert_eq!(res, Some(true));
    setup
        .contract
        .assert_acl_is_admin(false, role, admin.id())
        .await;

    // Revoking a role for which the account isn't admin returns `Some(false)`.
    let revoker = setup.new_account_as_admin(&[role]).await?;
    let account = setup.worker.dev_create_account().await?;
    let res = setup
        .contract
        .acl_revoke_admin(&revoker, role, account.id())
        .await?;
    assert_eq!(res, Some(false));

    Ok(())
}

#[tokio::test]
async fn test_acl_renounce_admin() -> anyhow::Result<()> {
    let setup = Setup::new().await?;
    let role = "Resetter";

    // An account which is isn't admin calls `acl_renounce_admin`.
    let res = setup
        .contract
        .acl_renounce_admin(&setup.account, role)
        .await?;
    assert!(!res);

    // An admin calls `acl_renounce_admin`.
    let admin = setup.new_account_as_admin(&[role]).await?;
    setup
        .contract
        .assert_acl_is_admin(true, role, admin.id())
        .await;
    let res = setup.contract.acl_renounce_admin(&admin, role).await?;
    assert!(res);
    setup
        .contract
        .assert_acl_is_admin(false, role, admin.id())
        .await;

    Ok(())
}

#[tokio::test]
async fn test_acl_revoke_admin_unchecked() -> anyhow::Result<()> {
    let setup = Setup::new().await?;
    let account = setup
        .new_account_as_admin(&["ByMax2Increaser", "Resetter"])
        .await?;

    setup
        .contract
        .assert_acl_is_admin(true, "ByMax2Increaser", account.id())
        .await;
    setup
        .contract
        .assert_acl_is_admin(true, "Resetter", account.id())
        .await;

    // Revoke admin permissions for one of the roles.
    let res = setup
        .contract
        .acl_revoke_admin_unchecked(setup.contract_account(), "ByMax2Increaser", account.id())
        .await?;
    assert_success_with(res, true);
    setup
        .contract
        .assert_acl_is_admin(false, "ByMax2Increaser", account.id())
        .await;
    setup
        .contract
        .assert_acl_is_admin(true, "Resetter", account.id())
        .await;

    // Revoke admin permissions for the other role too.
    let res = setup
        .contract
        .acl_revoke_admin_unchecked(setup.contract_account(), "Resetter", account.id())
        .await?;
    assert_success_with(res, true);
    setup
        .contract
        .assert_acl_is_admin(false, "ByMax2Increaser", account.id())
        .await;
    setup
        .contract
        .assert_acl_is_admin(false, "Resetter", account.id())
        .await;

    // Revoking behaves as expected if the permission is not present.
    let res = setup
        .contract
        .acl_revoke_admin_unchecked(setup.contract_account(), "Resetter", account.id())
        .await?;
    assert_success_with(res, false);

    Ok(())
}

#[tokio::test]
async fn test_acl_has_role() -> anyhow::Result<()> {
    let Setup {
        contract, account, ..
    } = Setup::new().await?;
    let contract_account = contract.contract().as_account();
    let role = "ByMax2Increaser";

    let has_role = contract.acl_has_role(&account, role, account.id()).await?;
    assert!(!has_role);

    contract
        .acl_grant_role_unchecked(contract_account, role, account.id())
        .await?
        .into_result()?;

    let has_role = contract.acl_has_role(&account, role, account.id()).await?;
    assert!(has_role);

    Ok(())
}

#[tokio::test]
#[allow(clippy::similar_names)]
async fn test_acl_grant_role() -> anyhow::Result<()> {
    let Setup {
        worker,
        contract,
        account,
        ..
    } = Setup::new().await?;
    let contract_account = contract.contract().as_account();
    let role = "ByMax3Increaser";

    let granter = account;
    let grantee = worker.dev_create_account().await?;

    // An account which isn't admin can't grant the role.
    contract
        .assert_acl_is_admin(false, role, granter.id())
        .await;
    let granted = contract
        .acl_grant_role(&granter, role, grantee.id())
        .await?;
    assert_eq!(granted, None);
    contract
        .assert_acl_has_role(false, role, grantee.id())
        .await;

    // Admin can grant the role.
    contract
        .acl_add_admin_unchecked(contract_account, role, granter.id())
        .await?
        .into_result()?;
    let granted = contract
        .acl_grant_role(&granter, role, grantee.id())
        .await?;
    assert_eq!(granted, Some(true));
    contract.assert_acl_has_role(true, role, grantee.id()).await;

    // Granting the role to an account which already is a grantee.
    let granted = contract
        .acl_grant_role(&granter, role, grantee.id())
        .await?;
    assert_eq!(granted, Some(false));

    Ok(())
}

#[tokio::test]
async fn test_acl_grant_role_unchecked() -> anyhow::Result<()> {
    let Setup {
        contract, account, ..
    } = Setup::new().await?;
    let contract_account = contract.contract().as_account();
    let role = "ByMax2Increaser";

    contract
        .assert_acl_has_role(false, role, account.id())
        .await;
    let res = contract
        .acl_grant_role_unchecked(contract_account, role, account.id())
        .await?;
    assert_success_with(res, true);
    contract.assert_acl_has_role(true, role, account.id()).await;

    // Granting a role again behaves as expected.
    let res = contract
        .acl_grant_role_unchecked(contract_account, role, account.id())
        .await?;
    assert_success_with(res, false);

    Ok(())
}

#[tokio::test]
async fn test_acl_revoke_role() -> anyhow::Result<()> {
    let setup = Setup::new().await?;
    let role = "ByMax3Increaser";
    let grantee = setup.new_account_with_roles(&[role]).await?;

    setup
        .contract
        .assert_acl_has_role(true, role, grantee.id())
        .await;

    // Revoke is a no-op if revoker is not an admin for the role.
    let revoker = setup.new_account_as_admin(&[]).await?;
    let res = setup
        .contract
        .acl_revoke_role(&revoker, role, grantee.id())
        .await?;
    assert_eq!(res, None);
    let revoker = setup.new_account_as_admin(&["ByMax2Increaser"]).await?;
    let res = setup
        .contract
        .acl_revoke_role(&revoker, role, grantee.id())
        .await?;
    assert_eq!(res, None);
    setup
        .contract
        .assert_acl_has_role(true, role, grantee.id())
        .await;

    // Revoke succeeds if the revoker is an admin for the role.
    let revoker = setup.new_account_as_admin(&[role]).await?;
    let res = setup
        .contract
        .acl_revoke_role(&revoker, role, grantee.id())
        .await?;
    assert_eq!(res, Some(true));
    setup
        .contract
        .assert_acl_has_role(false, role, grantee.id())
        .await;

    // Revoking a role that isn't granted returns `Some(false)`.
    let revoker = setup.new_account_as_admin(&[role]).await?;
    let account = setup.worker.dev_create_account().await?;
    let res = setup
        .contract
        .acl_revoke_role(&revoker, role, account.id())
        .await?;
    assert_eq!(res, Some(false));

    Ok(())
}

#[tokio::test]
async fn test_acl_renounce_role() -> anyhow::Result<()> {
    let setup = Setup::new().await?;
    let role = "Resetter";

    // An account which is isn't grantee calls `acl_renounce_role`.
    let res = setup
        .contract
        .acl_renounce_role(&setup.account, role)
        .await?;
    assert!(!res);

    // A grantee calls `acl_renounce_admin`.
    let grantee = setup.new_account_with_roles(&[role]).await?;
    setup
        .contract
        .assert_acl_has_role(true, role, grantee.id())
        .await;
    let res = setup.contract.acl_renounce_role(&grantee, role).await?;
    assert!(res);
    setup
        .contract
        .assert_acl_has_role(false, role, grantee.id())
        .await;

    Ok(())
}

#[tokio::test]
async fn test_acl_revoke_role_unchecked() -> anyhow::Result<()> {
    let setup = Setup::new().await?;
    let account = setup
        .new_account_with_roles(&["ByMax2Increaser", "Resetter"])
        .await?;

    setup
        .contract
        .assert_acl_has_role(true, "ByMax2Increaser", account.id())
        .await;
    setup
        .contract
        .assert_acl_has_role(true, "Resetter", account.id())
        .await;

    // Revoke one of the roles.
    let res = setup
        .contract
        .acl_revoke_role_unchecked(setup.contract_account(), "ByMax2Increaser", account.id())
        .await?;
    assert_success_with(res, true);
    setup
        .contract
        .assert_acl_has_role(false, "ByMax2Increaser", account.id())
        .await;
    setup
        .contract
        .assert_acl_has_role(true, "Resetter", account.id())
        .await;

    // Revoke the other role too.
    let res = setup
        .contract
        .acl_revoke_role_unchecked(setup.contract_account(), "Resetter", account.id())
        .await?;
    assert_success_with(res, true);
    setup
        .contract
        .assert_acl_has_role(false, "ByMax2Increaser", account.id())
        .await;
    setup
        .contract
        .assert_acl_has_role(false, "Resetter", account.id())
        .await;

    // Revoking behaves as expected if the role is not granted.
    let res = setup
        .contract
        .acl_revoke_role_unchecked(setup.contract_account(), "Resetter", account.id())
        .await?;
    assert_success_with(res, false);

    Ok(())
}

#[tokio::test]
async fn test_attribute_access_control_any() -> anyhow::Result<()> {
    let setup = Setup::new().await?;
    let raw_contract = setup.contract.contract();
    let method_name = "increase_2";
    let allowed_roles = vec!["ByMax2Increaser".to_string(), "ByMax3Increaser".to_string()];

    // Account without any of the required permissions is restricted.
    let account = setup.new_account_with_roles(&[]).await?;
    let res = call_increase_2(raw_contract, &account).await?;
    assert_insufficient_acl_permissions(res, method_name, &allowed_roles);
    let account = setup.new_account_with_roles(&["Resetter"]).await?;
    let res = call_increase_2(raw_contract, &account).await?;
    assert_insufficient_acl_permissions(res, method_name, &allowed_roles);

    // A super-admin which has not been granted the role is restricted.
    let super_admin = setup.new_super_admin_account().await?;
    let res = call_increase_2(raw_contract, &super_admin).await?;
    assert_insufficient_acl_permissions(res, method_name, &allowed_roles);

    // An admin for a permitted role is restricted (no grantee of role itself).
    let admin = setup.new_account_as_admin(&["ByMax2Increaser"]).await?;
    let res = call_increase_2(raw_contract, &admin).await?;
    assert_insufficient_acl_permissions(res, method_name, &allowed_roles);

    // Account with one of the required permissions succeeds.
    let account = setup.new_account_with_roles(&["ByMax2Increaser"]).await?;
    let res = call_increase_2(raw_contract, &account).await?;
    assert_success_with(res, 2);
    let account = setup.new_account_with_roles(&["ByMax3Increaser"]).await?;
    let res = call_increase_2(raw_contract, &account).await?;
    assert_success_with(res, 4);
    let account = setup
        .new_account_with_roles(&["ByMax2Increaser", "Resetter"])
        .await?;
    let res = call_increase_2(raw_contract, &account).await?;
    assert_success_with(res, 6);

    // Account with both permissions succeeds.
    let account = setup
        .new_account_with_roles(&["ByMax2Increaser", "ByMax3Increaser"])
        .await?;
    let res = call_increase_2(raw_contract, &account).await?;
    assert_success_with(res, 8);
    let account = setup
        .new_account_with_roles(&["ByMax2Increaser", "ByMax3Increaser", "Resetter"])
        .await?;
    let res = call_increase_2(raw_contract, &account).await?;
    assert_success_with(res, 10);

    Ok(())
}

#[tokio::test]
async fn test_acl_init_super_admin_is_private() -> anyhow::Result<()> {
    let Setup {
        contract, account, ..
    } = Setup::new().await?;
    let res = contract
        .acl_init_super_admin(&account, account.id())
        .await?;
    assert_private_method_failure(res, "acl_init_super_admin");
    Ok(())
}

#[tokio::test]
async fn test_acl_get_super_admins() -> anyhow::Result<()> {
    let setup = Setup::new().await?;

    let super_admin_ids = vec![
        setup.new_super_admin_account().await?,
        setup.new_super_admin_account().await?,
        setup.new_super_admin_account().await?,
    ]
    .iter()
    .map(|account| account.id().clone())
    .collect::<Vec<_>>();

    // Behaves as expected for limit = 0.
    let actual = setup
        .contract
        .acl_get_super_admins(&setup.account, 0, 0)
        .await?;
    assert!(actual.is_empty());

    // Skip outside of the number of existing super-admins.
    let n = u64::try_from(super_admin_ids.len()).unwrap();
    let actual = setup
        .contract
        .acl_get_super_admins(&setup.account, n, 1)
        .await?;
    assert!(actual.is_empty());

    // Retrieve super-admins with step size 1.
    for i in 0..3 {
        let actual = setup
            .contract
            .acl_get_super_admins(&setup.account, i, 1)
            .await?;
        let i = usize::try_from(i).unwrap();
        let expected = super_admin_ids[i..=i].to_vec();
        assert_eq!(actual, expected, "Mismatch at position {i}");
    }

    // Retrieve super-admins with step size 2.
    let actual = setup
        .contract
        .acl_get_super_admins(&setup.account, 0, 2)
        .await?;
    let expected = super_admin_ids[0..2].to_vec();
    assert_eq!(actual, expected);
    let actual = setup
        .contract
        .acl_get_super_admins(&setup.account, 2, 2)
        .await?;
    let expected = vec![super_admin_ids[2].clone()];
    assert_eq!(actual, expected);

    // Retrieve all super-admins at once.
    let actual = setup
        .contract
        .acl_get_super_admins(&setup.account, 0, 3)
        .await?;
    assert_eq!(actual, super_admin_ids);

    // Limit larger than the number of existing super-admins.
    let actual = setup
        .contract
        .acl_get_super_admins(&setup.account, 0, 4)
        .await?;
    assert_eq!(actual, super_admin_ids);

    Ok(())
}

#[tokio::test]
async fn test_acl_get_admins() -> anyhow::Result<()> {
    let setup = Setup::new().await?;
    let role = "ByMax3Increaser";

    let admin_ids = vec![
        setup.new_account_as_admin(&[role]).await?,
        setup.new_account_as_admin(&[role]).await?,
        setup.new_account_as_admin(&[role]).await?,
    ]
    .iter()
    .map(|account| account.id().clone())
    .collect::<Vec<_>>();

    // Behaves as expected for limit = 0.
    let actual = setup
        .contract
        .acl_get_admins(&setup.account, role, 0, 0)
        .await?;
    assert!(actual.is_empty());

    // Skip outside of the number of existing admins.
    let n = u64::try_from(admin_ids.len()).unwrap();
    let actual = setup
        .contract
        .acl_get_admins(&setup.account, role, n, 1)
        .await?;
    assert!(actual.is_empty());

    // Retrieve admins with step size 1.
    for i in 0..3 {
        let actual = setup
            .contract
            .acl_get_admins(&setup.account, role, i, 1)
            .await?;
        let i = usize::try_from(i).unwrap();
        let expected = admin_ids[i..=i].to_vec();
        assert_eq!(actual, expected, "Mismatch at position {i}");
    }

    // Retrieve admins with step size 2.
    let actual = setup
        .contract
        .acl_get_admins(&setup.account, role, 0, 2)
        .await?;
    let expected = admin_ids[0..2].to_vec();
    assert_eq!(actual, expected);
    let actual = setup
        .contract
        .acl_get_admins(&setup.account, role, 2, 2)
        .await?;
    let expected = vec![admin_ids[2].clone()];
    assert_eq!(actual, expected);

    // Retrieve all admins at once.
    let actual = setup
        .contract
        .acl_get_admins(&setup.account, role, 0, 3)
        .await?;
    assert_eq!(actual, admin_ids);

    // Limit larger than the number of existing admins.
    let actual = setup
        .contract
        .acl_get_admins(&setup.account, role, 0, 4)
        .await?;
    assert_eq!(actual, admin_ids);

    Ok(())
}

#[tokio::test]
async fn test_acl_get_grantees() -> anyhow::Result<()> {
    let setup = Setup::new().await?;
    let role = "ByMax2Increaser";

    let grantee_ids = vec![
        setup.new_account_with_roles(&[role]).await?,
        setup.new_account_with_roles(&[role]).await?,
        setup.new_account_with_roles(&[role]).await?,
    ]
    .iter()
    .map(|account| account.id().clone())
    .collect::<Vec<_>>();

    // Behaves as expected for limit = 0.
    let actual = setup
        .contract
        .acl_get_grantees(&setup.account, role, 0, 0)
        .await?;
    assert!(actual.is_empty());

    // Skip outside of the number of existing grantees.
    let n = u64::try_from(grantee_ids.len()).unwrap();
    let actual = setup
        .contract
        .acl_get_grantees(&setup.account, role, n, 1)
        .await?;
    assert!(actual.is_empty());

    // Retrieve grantees with step size 1.
    for i in 0..3 {
        let actual = setup
            .contract
            .acl_get_grantees(&setup.account, role, i, 1)
            .await?;
        let i = usize::try_from(i).unwrap();
        let expected = grantee_ids[i..=i].to_vec();
        assert_eq!(actual, expected, "Mismatch at position {i}");
    }

    // Retrieve grantees with step size 2.
    let actual = setup
        .contract
        .acl_get_grantees(&setup.account, role, 0, 2)
        .await?;
    let expected = grantee_ids[0..2].to_vec();
    assert_eq!(actual, expected);
    let actual = setup
        .contract
        .acl_get_grantees(&setup.account, role, 2, 2)
        .await?;
    let expected = vec![grantee_ids[2].clone()];
    assert_eq!(actual, expected);

    // Retrieve all grantees at once.
    let actual = setup
        .contract
        .acl_get_grantees(&setup.account, role, 0, 3)
        .await?;
    assert_eq!(actual, grantee_ids);

    // Limit larger than the number of existing grantees.
    let actual = setup
        .contract
        .acl_get_grantees(&setup.account, role, 0, 4)
        .await?;
    assert_eq!(actual, grantee_ids);

    Ok(())
}

#[tokio::test]
async fn test_acl_get_permissioned_accounts() -> anyhow::Result<()> {
    let setup = Setup::new().await?;

    // Verify returned `PermissionedAccounts` are empty when there are no outstanding permissions.
    let permissioned_accounts = setup
        .contract
        .acl_get_permissioned_accounts(&setup.account)
        .await?;
    let mut expected = new_permissioned_accounts();
    assert_permissioned_account_equivalence(&permissioned_accounts, &expected);

    // Add a super admin to the contract's Acl.
    let super_admin = setup.worker.dev_create_account().await?;
    let res = setup
        .contract
        .acl_init_super_admin(setup.contract_account(), super_admin.id())
        .await?;
    assert_success_with(res, true);

    // Add admins and grantees to the contract's Acl.
    let admin_0 = setup.new_account_as_admin(&[ALL_ROLES[0]]).await?;
    let admin_2 = setup.new_account_as_admin(&[ALL_ROLES[2]]).await?;
    let grantee_1_a = setup.new_account_with_roles(&[ALL_ROLES[1]]).await?;
    let grantee_1_b = setup.new_account_with_roles(&[ALL_ROLES[1]]).await?;

    // Insert ids added to contract's Acl into `expected`.
    expected
        .super_admins
        .push(as_sdk_account_id(super_admin.id()));
    expected
        .roles
        .get_mut(ALL_ROLES[0])
        .unwrap()
        .admins
        .push(as_sdk_account_id(admin_0.id()));
    expected
        .roles
        .get_mut(ALL_ROLES[1])
        .unwrap()
        .grantees
        .extend([
            as_sdk_account_id(grantee_1_a.id()),
            as_sdk_account_id(grantee_1_b.id()),
        ]);
    expected
        .roles
        .get_mut(ALL_ROLES[2])
        .unwrap()
        .admins
        .push(as_sdk_account_id(admin_2.id()));

    // Verify returned `PermissionedAccounts` when there are outstanding permissions.
    let permissioned_accounts = setup
        .contract
        .acl_get_permissioned_accounts(&setup.account)
        .await?;
    assert_permissioned_account_equivalence(&permissioned_accounts, &expected);

    Ok(())
}

#[tokio::test]
async fn test_acl_add_super_admin_unchecked_is_private() -> anyhow::Result<()> {
    let Setup {
        contract, account, ..
    } = Setup::new().await?;
    let res = contract
        .acl_add_super_admin_unchecked(&account, account.id())
        .await?;
    assert_private_method_failure(res, "acl_add_super_admin_unchecked");
    Ok(())
}

#[tokio::test]
async fn test_acl_revoke_super_admin_unchecked_is_private() -> anyhow::Result<()> {
    let Setup {
        contract, account, ..
    } = Setup::new().await?;
    let res = contract
        .acl_revoke_super_admin_unchecked(&account, account.id())
        .await?;
    assert_private_method_failure(res, "acl_revoke_super_admin_unchecked");
    Ok(())
}

#[tokio::test]
async fn test_acl_add_admin_unchecked_is_private() -> anyhow::Result<()> {
    let Setup {
        contract, account, ..
    } = Setup::new().await?;
    let res = contract
        .acl_add_admin_unchecked(&account, "ByMax2Increaser", account.id())
        .await?;
    assert_private_method_failure(res, "acl_add_admin_unchecked");
    Ok(())
}

#[tokio::test]
async fn test_acl_revoke_admin_unchecked_is_private() -> anyhow::Result<()> {
    let Setup {
        contract, account, ..
    } = Setup::new().await?;
    let res = contract
        .acl_revoke_admin_unchecked(&account, "ByMax2Increaser", account.id())
        .await?;
    assert_private_method_failure(res, "acl_revoke_admin_unchecked");
    Ok(())
}

#[tokio::test]
async fn test_acl_grant_role_unchecked_is_private() -> anyhow::Result<()> {
    let Setup {
        contract, account, ..
    } = Setup::new().await?;
    let res = contract
        .acl_grant_role_unchecked(&account, "ByMax2Increaser", account.id())
        .await?;
    assert_private_method_failure(res, "acl_grant_role_unchecked");
    Ok(())
}

#[tokio::test]
async fn test_acl_revoke_role_unchecked_is_private() -> anyhow::Result<()> {
    let Setup {
        contract, account, ..
    } = Setup::new().await?;
    let res = contract
        .acl_revoke_role_unchecked(&account, "ByMax2Increaser", account.id())
        .await?;
    assert_private_method_failure(res, "acl_revoke_role_unchecked");
    Ok(())
}

const WASM_V_0_2_0_FILEPATH: &str = "tests/data/access_controllable_v_0_2_0.wasm";

#[tokio::test]
async fn test_upgrade_storage() -> anyhow::Result<()> {
    let role = "ByMax2Increaser";
    let role2 = "Resetter";

    let old_wasm = std::fs::read(WASM_V_0_2_0_FILEPATH)?;
    let Setup {
        contract, account, ..
    } = Setup::new_with_wasm(old_wasm).await?;

    let contract_account = contract.contract().as_account();

    let _ = contract
        .acl_init_super_admin(contract_account, account.id())
        .await?;

    let result = contract
        .acl_grant_role(&account, role, account.id())
        .await?;
    assert!(result.is_some());
    assert!(result.unwrap());

    let result = contract
        .acl_grant_role(&account, role2, account.id())
        .await?;
    assert!(result.is_some());
    assert!(result.unwrap());

    let admin_account_id: AccountId = "alice.near".parse().unwrap();
    let added = contract
        .acl_add_admin(&account, role, &admin_account_id)
        .await?;
    assert!(added.is_some());
    assert!(added.unwrap());

    let new_wasm =
        common::repo::compile_project(Path::new(PROJECT_PATH), "access_controllable").await?;

    // New version
    let contract = contract
        .contract()
        .as_account()
        .deploy(&new_wasm)
        .await
        .unwrap()
        .result;
    let contract = AccessControllableContract::new(contract);

    let has_role = contract.acl_has_role(&account, role, account.id()).await?;
    assert!(has_role);

    let has_role = contract.acl_has_role(&account, role2, account.id()).await?;
    assert!(has_role);

    let is_admin = contract
        .acl_is_admin(&account, role, &admin_account_id)
        .await?;
    assert!(is_admin);

    let is_super_admin = contract.acl_is_super_admin(&account, account.id()).await?;
    assert!(is_super_admin);

    Ok(())
}
