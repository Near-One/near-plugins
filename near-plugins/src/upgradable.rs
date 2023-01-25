//! # Upgradable
//!
//! Upgradable trait inspired by [NEP123](https://github.com/near/NEPs/pull/123).
//!
//! To upgrade the contract, first the code needs to be staged, and then it can be deployed.
//!
//! ## Default implementation
//!
//! Only owner or self can call [`Upgradable::up_stage_code`] and [`Upgradable::up_deploy_code`].
//!
//! There is no timer or staging duration implemented by default.
//!
//! ## Permissions
//!
//! Only an authorized account is allowed to call [`Upgradable::up_stage_code`] and
//! [`Upgradable::up_deploy_code`]. There may be several reasons to protect `deploy_code`, one of
//! them being an upgrade that requires migration.
//!
//! ## Upgrade that requires state migration
//!
//! If a contract upgrade requires state migration, it is recommended to execute the upgrade in a
//! batch transaction that calls [`Upgradable::up_deploy_code`] followed by the migration method. To
//! verify the migration was successful, in addition, a function that deserializes contract state
//! might be called.
//!
//! Note that even if above actions are executed in a batch transaction, the staged code is still
//! deployed in case the migration fails. This is due to the [asynchronous nature] of NEAR and the
//! deployment ultimately being executed in a separate promise that is not affected by the outcome
//! of the batch transaction. Still, a batch transaction helps to minimize the time between the
//! deployment of new code and the migration of state.
//!
//! ## Stale staged code
//!
//! After the code is deployed, it should be removed from staging. This will prevent old code with a
//! security vulnerability to be deployed.
//!
//! ## Upgrading code that contains a security vulnerability
//!
//! Once code is staged for an upgrade, it is publicly visible via [`Upgradable::up_staged_code`].
//! Staged code that fixes a security vulnerability might be discovered by an attacker who then
//! exploits the vulnerability before its fix is deployed.
//!
//! To avoid that, the upgrade can be executed by calling [`Upgradable::up_stage_code`] and
//! [`Upgradable::up_deploy_code`] in a [batch transaction]. Since [`Upgradable::up_deploy_code`]
//! returns a promise that ultimately deploys the new contract code, a theoretical risk remains.
//! However, the [time between scheduling and execution] of a promise hardly allows an attacker to
//! exploit a vulnerability: they would have to retrieve the bytes of the staged code, reverse
//! engineer the new contract, build an exploit and finally execute it. Therefore, we consider that
//! risk of an exploit in case of a batched upgrade negligible.
//!
//! Another defense mechanism is staging encrypted code, though this requires your own
//! implementation of the trait `Upgradable`. The default implementation provided by
//! `near-plugins-derive` does not support it.
//!
//! [asynchronous nature]: https://docs.near.org/concepts/basics/transactions/overview
//! [batch transaction]: https://docs.near.org/concepts/basics/transactions/overview
//! [time between scheduling and execution]: https://docs.near.org/sdk/rust/promises/intro
use crate::events::{AsEvent, EventMetadata};
use near_sdk::{AccountId, CryptoHash, Promise};
use serde::Serialize;

/// Trait describing the functionality of the _Upgradable_ plugin.
pub trait Upgradable {
    /// Key of storage slot to save the staged code.
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
    /// The account which staged the code.
    by: AccountId,
    /// The hash of the code that was staged.
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
    /// The account that deployed the code.
    by: AccountId,
    /// The hash of the code that was deployed.
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
