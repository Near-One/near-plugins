use near_sdk::serde_json::json;
use std::collections::HashSet;
use workspaces::result::ExecutionFinalResult;
use workspaces::{Account, Contract};

/// Wrapper for a contract that is `#[pausable]`. It allows implementing helpers
/// for calling contract methods.
pub struct PausableContract {
    contract: Contract,
}

impl PausableContract {
    pub fn new(contract: Contract) -> Self {
        Self { contract }
    }

    pub fn contract(&self) -> &Contract {
        &self.contract
    }

    pub async fn pa_is_paused(&self, caller: &Account, key: &str) -> anyhow::Result<bool> {
        let res = caller
            .call(self.contract.id(), "pa_is_paused")
            .args_json(json!({
                "key": key,
            }))
            .view()
            .await?;
        Ok(res.json::<bool>()?)
    }

    pub async fn pa_pause_feature(
        &self,
        caller: &Account,
        key: &str,
    ) -> workspaces::Result<ExecutionFinalResult> {
        caller
            .call(self.contract.id(), "pa_pause_feature")
            .args_json(json!({ "key": key }))
            .max_gas()
            .transact()
            .await
    }

    pub async fn pa_unpause_feature(
        &self,
        caller: &Account,
        key: &str,
    ) -> workspaces::Result<ExecutionFinalResult> {
        caller
            .call(self.contract.id(), "pa_unpause_feature")
            .args_json(json!({ "key": key }))
            .max_gas()
            .transact()
            .await
    }

    pub async fn pa_all_paused(&self, caller: &Account) -> anyhow::Result<Option<HashSet<String>>> {
        let res = caller
            .call(self.contract.id(), "pa_all_paused")
            .view()
            .await?;
        Ok(res.json::<Option<HashSet<String>>>()?)
    }
}
