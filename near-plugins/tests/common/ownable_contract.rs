use near_sdk::serde_json::json;
use workspaces::result::ExecutionFinalResult;
use workspaces::{Account, AccountId, Contract};

/// Wrapper for a contract that is `#[ownable]`. It allows implementing helpers for calling contract
/// methods.
pub struct OwnableContract {
    contract: Contract,
}

impl OwnableContract {
    pub fn new(contract: Contract) -> Self {
        Self { contract }
    }

    pub fn contract(&self) -> &Contract {
        &self.contract
    }

    pub async fn owner_get(&self, caller: &Account) -> anyhow::Result<Option<AccountId>> {
        let res = caller.call(self.contract.id(), "owner_get").view().await?;
        Ok(res.json::<Option<AccountId>>()?)
    }

    pub async fn owner_set(
        &self,
        caller: &Account,
        owner: Option<AccountId>,
    ) -> workspaces::Result<ExecutionFinalResult> {
        caller
            .call(self.contract.id(), "owner_set")
            .args_json(json!({ "owner": owner }))
            .max_gas()
            .transact()
            .await
    }

    pub async fn owner_is(&self, caller: &Account) -> anyhow::Result<bool> {
        let res = caller
            .call(self.contract.id(), "owner_is")
            .max_gas()
            .transact()
            .await?;
        Ok(res.json::<bool>()?)
    }
}
