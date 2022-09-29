use near_sdk::serde_json::json;
use workspaces::result::ExecutionFinalResult;
use workspaces::{Account, AccountId, Contract};

/// Specifies who calls a method on the contract.
#[derive(Clone)]
pub enum Caller {
    /// The contract itself.
    Contract,
    /// The provided account.
    Account(Account),
}

impl From<Account> for Caller {
    fn from(account: Account) -> Self {
        Self::Account(account)
    }
}

/// Wrapper for a contract that is `#[access_controllable]`. It allows
/// implementing helpers for calling contract methods.
pub struct AccessControllableContract {
    contract: Contract,
}

impl AccessControllableContract {
    pub fn new(contract: Contract) -> Self {
        Self { contract }
    }

    pub fn contract(&self) -> &Contract {
        &self.contract
    }

    fn account(&self, caller: Caller) -> Account {
        match caller {
            Caller::Contract => self.contract.as_account().clone(),
            Caller::Account(account) => account,
        }
    }

    pub async fn acl_is_admin(
        &self,
        caller: Caller,
        role: &str,
        account_id: &AccountId,
    ) -> anyhow::Result<bool> {
        let res = self
            .account(caller)
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
            .acl_is_admin(Caller::Contract, role, account_id)
            .await
            .unwrap();
        assert_eq!(is_admin, expected);
    }

    pub async fn acl_add_admin(
        &self,
        caller: Caller,
        role: &str,
        account_id: &AccountId,
    ) -> anyhow::Result<Option<bool>> {
        let res = self
            .account(caller)
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
        caller: Caller,
        role: &str,
        account_id: &AccountId,
    ) -> workspaces::Result<ExecutionFinalResult> {
        self.account(caller)
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
        caller: Caller,
        role: &str,
        account_id: &AccountId,
    ) -> anyhow::Result<Option<bool>> {
        let res = self
            .account(caller)
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

    pub async fn acl_renounce_admin(&self, caller: Caller, role: &str) -> anyhow::Result<bool> {
        let res = self
            .account(caller)
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
        caller: Caller,
        role: &str,
        account_id: &AccountId,
    ) -> workspaces::Result<ExecutionFinalResult> {
        self.account(caller)
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
        caller: Caller,
        role: &str,
        account_id: &AccountId,
    ) -> anyhow::Result<bool> {
        let res = self
            .account(caller)
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
            .acl_has_role(Caller::Contract, role, account_id)
            .await
            .unwrap();
        assert_eq!(has_role, expected);
    }

    pub async fn acl_grant_role_unchecked(
        &self,
        caller: Caller,
        role: &str,
        account_id: &AccountId,
    ) -> workspaces::Result<ExecutionFinalResult> {
        self.account(caller)
            .call(self.contract.id(), "acl_grant_role_unchecked")
            .args_json(json!({
                "role": role,
                "account_id": account_id,
            }))
            .max_gas()
            .transact()
            .await
    }
}
