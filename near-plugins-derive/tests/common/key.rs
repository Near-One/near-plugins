use near_workspaces::result::ExecutionFinalResult;
use near_workspaces::types::{AccessKeyInfo, PublicKey};
use near_workspaces::{Account, AccountId, Contract};

/// Creates a transaction signed by `signer` to delete `key` from `contract`.
pub async fn delete_access_key(
    signer: &Account,
    contract: &AccountId,
    key: PublicKey,
) -> near_workspaces::Result<ExecutionFinalResult> {
    signer.batch(contract).delete_key(key).transact().await
}

/// Panics if access key info cannot be retrieved.
pub async fn get_access_key_infos(contract: &Contract) -> Vec<AccessKeyInfo> {
    contract
        .view_access_keys()
        .await
        .expect("Should retrieve access keys")
}
