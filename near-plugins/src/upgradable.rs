/// # Upgrdable:
///
/// Upgradable implementation inspired by [NEP123](https://github.com/near/NEPs/pull/123).
///
/// To upgrade the contract, first the code needs to be staged, and then it can be deployed.
///
/// ## Default implementation:
///
/// Only owner or self can call `stage_code` and `deploy_code`.
/// There is no timer or staging duration implemented by default.
///
/// ## Security concerns:
///
/// Only authorized account is allowed to call `stage_code` and `deploy_code`. There may be several
/// reasons to protect `deploy_code`. One such reason to keep in mind, is when the code is upgraded
/// in such a way that requires some sort of migration or initialization. In that case, it is
/// recommended to run a batched transaction where `deploy_code` is called first, and then a
/// function that executes the migration or initialization.
///
/// After the code is deployed, it should be removed from staging. This will prevent an old code
/// with a security vulnerability to be deployed, in case it was upgraded using other mechanism.
use near_sdk::{AccountId, CryptoHash, Promise};

pub trait Upgradable {
    /// Key of storage slot to save the current owner.
    /// By default b"__CODE__" is used.
    fn up_storage_key(&self) -> Vec<u8>;

    /// Allows authorized account to stage some code to be potentially deployed later.
    /// If a previous code was staged but not deployed, it is discarded.
    fn up_stage_code(&mut self, code: Vec<u8>);

    /// Returns staged code.
    fn up_staged_code(&self) -> Option<Vec<u8>>;

    /// Returns hash of the staged code
    fn up_staged_code_hash(&self) -> Option<CryptoHash>;

    /// Allows authorized account to deploy staged code. If no code is staged the method fails.
    fn up_deploy_code(&mut self) -> Promise;
}

/// Event emitted when the code is staged
struct StageCode {
    by: AccountId,
    code_hash: CryptoHash,
}

/// Event emitted when the code is deployed
struct DeployCode {
    by: AccountId,
    code_hash: CryptoHash,
}
