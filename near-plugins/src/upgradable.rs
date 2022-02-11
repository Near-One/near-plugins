//! # Upgradable:
//!
//! Upgradable implementation inspired by [NEP123](https://github.com/near/NEPs/pull/123).
//!
//! To upgrade the contract, first the code needs to be staged, and then it can be deployed.
//!
//! ## Default implementation:
//!
//! Only owner or self can call `stage_code` and `deploy_code`.
//! There is no timer or staging duration implemented by default.
//!
//! ## Security concerns:
//!
//! Only authorized account is allowed to call `stage_code` and `deploy_code`. There may be several
//! reasons to protect `deploy_code`. One such reason to keep in mind, is when the code is upgraded
//! in such a way that requires some sort of migration or initialization. In that case, it is
//! recommended to run a batched transaction where `deploy_code` is called first, and then a
//! function that executes the migration or initialization.
//!
//! After the code is deployed, it should be removed from staging. This will prevent an old code
//! with a security vulnerability to be deployed, in case it was upgraded using other mechanism.
use crate::events::{AsEvent, EventMetadata};
use near_sdk::{AccountId, CryptoHash, Promise};
use serde::Serialize;

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
#[derive(Serialize, Clone)]
struct StageCode {
    by: AccountId,
    code_hash: CryptoHash,
}

impl AsEvent<StageCode> for StageCode {
    fn metadata(&self) -> EventMetadata<StageCode> {
        EventMetadata {
            standard: "Upgradable".to_string(),
            version: "1.0.0".to_string(),
            event: "stage_code".to_string(),
            data: Some(self.clone()),
        }
    }
}

/// Event emitted when the code is deployed
#[derive(Serialize, Clone)]
struct DeployCode {
    by: AccountId,
    code_hash: CryptoHash,
}

impl AsEvent<DeployCode> for DeployCode {
    fn metadata(&self) -> EventMetadata<DeployCode> {
        EventMetadata {
            standard: "Upgradable".to_string(),
            version: "1.0.0".to_string(),
            event: "deploy_code".to_string(),
            data: Some(self.clone()),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests {
    // TODO: Make simulation test that verifies code is deployed
    use crate as near_plugins;
    use crate::test_utils::get_context;
    use crate::{Ownable, Upgradable};
    use near_sdk::env::sha256;
    use near_sdk::{near_bindgen, testing_env, VMContext};
    use std::convert::TryInto;

    #[near_bindgen]
    #[derive(Ownable, Upgradable)]
    struct Counter;

    #[near_bindgen]
    impl Counter {
        /// Specify the owner of the contract in the constructor
        #[init]
        fn new() -> Self {
            let mut contract = Self {};
            contract.owner_set(Some(near_sdk::env::predecessor_account_id()));
            contract
        }
    }

    /// Setup basic account. Owner of the account is `eli.test`
    fn setup_basic() -> (Counter, VMContext) {
        let ctx = get_context();
        testing_env!(ctx.clone());
        let mut counter = Counter::new();
        counter.owner_set(Some("eli.test".to_string().try_into().unwrap()));
        (counter, ctx)
    }

    #[test]
    #[should_panic(expected = r#"Ownable: Method must be called from owner"#)]
    fn test_stage_code_not_owner() {
        let (mut counter, _) = setup_basic();
        counter.up_stage_code(vec![1]);
    }

    #[test]
    fn test_stage_code() {
        let (mut counter, mut ctx) = setup_basic();

        ctx.predecessor_account_id = "eli.test".to_string().try_into().unwrap();
        testing_env!(ctx);

        assert_eq!(counter.up_staged_code(), None);
        counter.up_stage_code(vec![1]);

        assert_eq!(counter.up_staged_code(), Some(vec![1]));

        assert_eq!(
            counter.up_staged_code_hash(),
            Some(sha256(vec![1].as_slice()).try_into().unwrap())
        );

        counter.up_deploy_code();
    }
}
