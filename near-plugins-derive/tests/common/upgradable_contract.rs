use near_plugins::upgradable::{FunctionCallArgs, UpgradableDurationStatus};

use near_sdk::serde_json::json;
use near_sdk::CryptoHash;
use near_sdk::Duration;
use near_workspaces::result::ExecutionFinalResult;
use near_workspaces::{Account, Contract};

/// Wrapper for a contract that derives `Upgradable`. It allows implementing helpers for calling
/// contract methods provided by `Upgradable`.
pub struct UpgradableContract {
    contract: Contract,
}

impl UpgradableContract {
    #[must_use]
    pub const fn new(contract: Contract) -> Self {
        Self { contract }
    }

    #[must_use]
    pub const fn contract(&self) -> &Contract {
        &self.contract
    }

    pub async fn up_get_delay_status(
        &self,
        caller: &Account,
    ) -> anyhow::Result<UpgradableDurationStatus> {
        let res = caller
            .call(self.contract.id(), "up_get_delay_status")
            .view()
            .await?;
        Ok(res.json::<UpgradableDurationStatus>()?)
    }

    pub async fn up_stage_code(
        &self,
        caller: &Account,
        code: Vec<u8>,
    ) -> near_workspaces::Result<ExecutionFinalResult> {
        caller
            .call(self.contract.id(), "up_stage_code")
            .args_borsh(code)
            .max_gas()
            .transact()
            .await
    }

    pub async fn up_staged_code(&self, caller: &Account) -> anyhow::Result<Option<Vec<u8>>> {
        let res = caller
            .call(self.contract.id(), "up_staged_code")
            .max_gas()
            .transact()
            .await?;
        Ok(res.borsh::<Option<Vec<u8>>>()?)
    }

    pub async fn up_staged_code_hash(
        &self,
        caller: &Account,
    ) -> anyhow::Result<Option<CryptoHash>> {
        let res = caller
            .call(self.contract.id(), "up_staged_code_hash")
            .max_gas()
            .transact()
            .await?;
        Ok(res.json::<Option<CryptoHash>>()?)
    }

    /// The `Promise` returned by trait method `up_deploy_code` is resolved in the `near_workspaces`
    /// transaction.
    pub async fn up_deploy_code(
        &self,
        caller: &Account,
        function_call_args: Option<FunctionCallArgs>,
    ) -> near_workspaces::Result<ExecutionFinalResult> {
        caller
            .call(self.contract.id(), "up_deploy_code")
            .args_json(json!({
                "function_call_args": function_call_args,
            }))
            .max_gas()
            .transact()
            .await
    }

    pub async fn up_init_staging_duration(
        &self,
        caller: &Account,
        staging_duration: Duration,
    ) -> near_workspaces::Result<ExecutionFinalResult> {
        caller
            .call(self.contract.id(), "up_init_staging_duration")
            .args_json(json!({ "staging_duration": staging_duration }))
            .max_gas()
            .transact()
            .await
    }

    pub async fn up_stage_update_staging_duration(
        &self,
        caller: &Account,
        staging_duration: Duration,
    ) -> near_workspaces::Result<ExecutionFinalResult> {
        caller
            .call(self.contract.id(), "up_stage_update_staging_duration")
            .args_json(json!({ "staging_duration": staging_duration }))
            .max_gas()
            .transact()
            .await
    }

    pub async fn up_apply_update_staging_duration(
        &self,
        caller: &Account,
    ) -> near_workspaces::Result<ExecutionFinalResult> {
        caller
            .call(self.contract.id(), "up_apply_update_staging_duration")
            .max_gas()
            .transact()
            .await
    }
}
