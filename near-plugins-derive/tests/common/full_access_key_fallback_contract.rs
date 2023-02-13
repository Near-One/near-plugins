use near_sdk::serde_json::json;
use near_sdk::PublicKey;
use workspaces::result::ExecutionFinalResult;
use workspaces::{Account, Contract};

/// Wrapper for a contract that uses `#[full_access_key_fallback]`. It allows implementing helpers
/// for calling contract methods.
pub struct FullAccessKeyFallbackContract {
    contract: Contract,
}

impl FullAccessKeyFallbackContract {
    pub fn new(contract: Contract) -> Self {
        Self { contract }
    }

    pub fn contract(&self) -> &Contract {
        &self.contract
    }

    /// The `Promise` returned by trait method `attach_full_access_key` is resolved in the
    /// workspaces transaction.
    pub async fn attach_full_access_key(
        &self,
        caller: &Account,
        public_key: PublicKey,
    ) -> workspaces::Result<ExecutionFinalResult> {
        caller
            .call(self.contract.id(), "attach_full_access_key")
            .args_json(json!({ "public_key": public_key }))
            .max_gas()
            .transact()
            .await
    }
}
