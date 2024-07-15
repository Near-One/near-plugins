use near_plugins::access_controllable::PermissionedAccounts;

use near_sdk::serde_json::json;
use near_workspaces::result::ExecutionFinalResult;
use near_workspaces::{Account, AccountId, Contract};

/// Wrapper for a contract that is `#[access_controllable]`. It allows
/// implementing helpers for calling contract methods.
pub struct AccessControllableContract {
    contract: Contract,
}

impl AccessControllableContract {
    #[must_use]
    pub const fn new(contract: Contract) -> Self {
        Self { contract }
    }

    #[must_use]
    pub const fn contract(&self) -> &Contract {
        &self.contract
    }

    pub async fn acl_role_variants(&self, caller: &Account) -> anyhow::Result<Vec<String>> {
        let res = caller
            .call(self.contract.id(), "acl_role_variants")
            .view()
            .await?;
        Ok(res.json::<Vec<String>>()?)
    }

    pub async fn acl_is_super_admin(
        &self,
        caller: &Account,
        account_id: &AccountId,
    ) -> anyhow::Result<bool> {
        let res = caller
            .call(self.contract.id(), "acl_is_super_admin")
            .args_json(json!({
                "account_id": account_id,
            }))
            .view()
            .await?;
        Ok(res.json::<bool>()?)
    }

    pub async fn assert_acl_is_super_admin(
        &self,
        expected: bool,
        caller: &Account,
        account_id: &AccountId,
    ) {
        let is_super_admin = self.acl_is_super_admin(caller, account_id).await.unwrap();
        assert_eq!(is_super_admin, expected);
    }

    pub async fn acl_init_super_admin(
        &self,
        caller: &Account,
        account_id: &AccountId,
    ) -> near_workspaces::Result<ExecutionFinalResult> {
        caller
            .call(self.contract.id(), "acl_init_super_admin")
            .args_json(json!({
                "account_id": account_id,
            }))
            .max_gas()
            .transact()
            .await
    }

    pub async fn acl_add_super_admin(
        &self,
        caller: &Account,
        account_id: &AccountId,
    ) -> anyhow::Result<Option<bool>> {
        let res = caller
            .call(self.contract.id(), "acl_add_super_admin")
            .args_json(json!({
                "account_id": account_id,
            }))
            .max_gas()
            .transact()
            .await?
            .into_result()?
            .json::<Option<bool>>()?;
        Ok(res)
    }

    pub async fn acl_add_super_admin_unchecked(
        &self,
        caller: &Account,
        account_id: &AccountId,
    ) -> near_workspaces::Result<ExecutionFinalResult> {
        caller
            .call(self.contract.id(), "acl_add_super_admin_unchecked")
            .args_json(json!({
                "account_id": account_id,
            }))
            .max_gas()
            .transact()
            .await
    }

    pub async fn acl_revoke_super_admin(
        &self,
        caller: &Account,
        account_id: &AccountId,
    ) -> anyhow::Result<Option<bool>> {
        let res = caller
            .call(self.contract.id(), "acl_revoke_super_admin")
            .args_json(json!({
                "account_id": account_id,
            }))
            .max_gas()
            .transact()
            .await?
            .into_result()?
            .json::<Option<bool>>()?;
        Ok(res)
    }

    pub async fn acl_transfer_super_admin(
        &self,
        caller: &Account,
        account_id: &AccountId,
    ) -> anyhow::Result<Option<bool>> {
        let res = caller
            .call(self.contract.id(), "acl_transfer_super_admin")
            .args_json(json!({
                "account_id": account_id,
            }))
            .max_gas()
            .transact()
            .await?
            .into_result()?
            .json::<Option<bool>>()?;
        Ok(res)
    }

    pub async fn acl_revoke_super_admin_unchecked(
        &self,
        caller: &Account,
        account_id: &AccountId,
    ) -> near_workspaces::Result<ExecutionFinalResult> {
        caller
            .call(self.contract.id(), "acl_revoke_super_admin_unchecked")
            .args_json(json!({
                "account_id": account_id,
            }))
            .max_gas()
            .transact()
            .await
    }

    pub async fn acl_is_admin(
        &self,
        caller: &Account,
        role: &str,
        account_id: &AccountId,
    ) -> anyhow::Result<bool> {
        let res = caller
            .call(self.contract.id(), "acl_is_admin")
            .args_json(json!({
                "role": role,
                "account_id": account_id,
            }))
            .view()
            .await?;
        Ok(res.json::<bool>()?)
    }

    pub async fn assert_acl_is_admin(&self, expected: bool, role: &str, account_id: &AccountId) {
        let is_admin = self
            .acl_is_admin(self.contract.as_account(), role, account_id)
            .await
            .unwrap();
        assert_eq!(is_admin, expected);
    }

    pub async fn acl_add_admin(
        &self,
        caller: &Account,
        role: &str,
        account_id: &AccountId,
    ) -> anyhow::Result<Option<bool>> {
        let res = caller
            .call(self.contract.id(), "acl_add_admin")
            .args_json(json!({
                "role": role,
                "account_id": account_id,
            }))
            .max_gas()
            .transact()
            .await?
            .into_result()?
            .json::<Option<bool>>()?;
        Ok(res)
    }

    pub async fn acl_add_admin_unchecked(
        &self,
        caller: &Account,
        role: &str,
        account_id: &AccountId,
    ) -> near_workspaces::Result<ExecutionFinalResult> {
        caller
            .call(self.contract.id(), "acl_add_admin_unchecked")
            .args_json(json!({
                "role": role,
                "account_id": account_id,
            }))
            .max_gas()
            .transact()
            .await
    }

    pub async fn acl_revoke_admin(
        &self,
        caller: &Account,
        role: &str,
        account_id: &AccountId,
    ) -> anyhow::Result<Option<bool>> {
        let res = caller
            .call(self.contract.id(), "acl_revoke_admin")
            .args_json(json!({
                "role": role,
                "account_id": account_id,
            }))
            .max_gas()
            .transact()
            .await?
            .into_result()?
            .json::<Option<bool>>()?;
        Ok(res)
    }

    pub async fn acl_renounce_admin(&self, caller: &Account, role: &str) -> anyhow::Result<bool> {
        let res = caller
            .call(self.contract.id(), "acl_renounce_admin")
            .args_json(json!({
                "role": role,
            }))
            .max_gas()
            .transact()
            .await?
            .into_result()?
            .json::<bool>()?;
        Ok(res)
    }

    pub async fn acl_revoke_admin_unchecked(
        &self,
        caller: &Account,
        role: &str,
        account_id: &AccountId,
    ) -> near_workspaces::Result<ExecutionFinalResult> {
        caller
            .call(self.contract.id(), "acl_revoke_admin_unchecked")
            .args_json(json!({
                "role": role,
                "account_id": account_id,
            }))
            .max_gas()
            .transact()
            .await
    }

    pub async fn acl_has_role(
        &self,
        caller: &Account,
        role: &str,
        account_id: &AccountId,
    ) -> anyhow::Result<bool> {
        let res = caller
            .call(self.contract.id(), "acl_has_role")
            .args_json(json!({
                "role": role,
                "account_id": account_id,
            }))
            .view()
            .await?;
        Ok(res.json::<bool>()?)
    }

    pub async fn assert_acl_has_role(&self, expected: bool, role: &str, account_id: &AccountId) {
        let has_role = self
            .acl_has_role(self.contract.as_account(), role, account_id)
            .await
            .unwrap();
        assert_eq!(has_role, expected);
    }

    pub async fn acl_grant_role(
        &self,
        caller: &Account,
        role: &str,
        account_id: &AccountId,
    ) -> anyhow::Result<Option<bool>> {
        let res = caller
            .call(self.contract.id(), "acl_grant_role")
            .args_json(json!({
                "role": role,
                "account_id": account_id,
            }))
            .max_gas()
            .transact()
            .await?
            .into_result()?
            .json::<Option<bool>>()?;
        Ok(res)
    }

    pub async fn acl_grant_role_unchecked(
        &self,
        caller: &Account,
        role: &str,
        account_id: &AccountId,
    ) -> near_workspaces::Result<ExecutionFinalResult> {
        caller
            .call(self.contract.id(), "acl_grant_role_unchecked")
            .args_json(json!({
                "role": role,
                "account_id": account_id,
            }))
            .max_gas()
            .transact()
            .await
    }

    pub async fn acl_revoke_role(
        &self,
        caller: &Account,
        role: &str,
        account_id: &AccountId,
    ) -> anyhow::Result<Option<bool>> {
        let res = caller
            .call(self.contract.id(), "acl_revoke_role")
            .args_json(json!({
                "role": role,
                "account_id": account_id,
            }))
            .max_gas()
            .transact()
            .await?
            .into_result()?
            .json::<Option<bool>>()?;
        Ok(res)
    }

    pub async fn acl_renounce_role(&self, caller: &Account, role: &str) -> anyhow::Result<bool> {
        let res = caller
            .call(self.contract.id(), "acl_renounce_role")
            .args_json(json!({
                "role": role,
            }))
            .max_gas()
            .transact()
            .await?
            .into_result()?
            .json::<bool>()?;
        Ok(res)
    }

    pub async fn acl_revoke_role_unchecked(
        &self,
        caller: &Account,
        role: &str,
        account_id: &AccountId,
    ) -> near_workspaces::Result<ExecutionFinalResult> {
        caller
            .call(self.contract.id(), "acl_revoke_role_unchecked")
            .args_json(json!({
                "role": role,
                "account_id": account_id,
            }))
            .max_gas()
            .transact()
            .await
    }

    pub async fn acl_get_super_admins(
        &self,
        caller: &Account,
        skip: u64,
        limit: u64,
    ) -> anyhow::Result<Vec<AccountId>> {
        let res = caller
            .call(self.contract.id(), "acl_get_super_admins")
            .args_json(json!({
                "skip": skip,
                "limit": limit,
            }))
            .max_gas()
            .transact()
            .await?
            .into_result()?
            .json::<Vec<AccountId>>()?;
        Ok(res)
    }

    pub async fn acl_get_admins(
        &self,
        caller: &Account,
        role: &str,
        skip: u64,
        limit: u64,
    ) -> anyhow::Result<Vec<AccountId>> {
        let res = caller
            .call(self.contract.id(), "acl_get_admins")
            .args_json(json!({
                "role": role,
                "skip": skip,
                "limit": limit,
            }))
            .max_gas()
            .transact()
            .await?
            .into_result()?
            .json::<Vec<AccountId>>()?;
        Ok(res)
    }

    pub async fn acl_get_grantees(
        &self,
        caller: &Account,
        role: &str,
        skip: u64,
        limit: u64,
    ) -> anyhow::Result<Vec<AccountId>> {
        let res = caller
            .call(self.contract.id(), "acl_get_grantees")
            .args_json(json!({
                "role": role,
                "skip": skip,
                "limit": limit,
            }))
            .max_gas()
            .transact()
            .await?
            .into_result()?
            .json::<Vec<AccountId>>()?;
        Ok(res)
    }

    pub async fn acl_get_permissioned_accounts(
        &self,
        caller: &Account,
    ) -> anyhow::Result<PermissionedAccounts> {
        let res = caller
            .call(self.contract.id(), "acl_get_permissioned_accounts")
            .view()
            .await?;
        Ok(res.json::<PermissionedAccounts>()?)
    }
}
