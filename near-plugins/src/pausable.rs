//! # Pausable:
//!
//! Trait which allows contracts to implement an emergency stop mechanism that can be triggered
//! by an authorized account. This authorized account can pause certain features which will
//! prevent some methods or behaviors to be executed. It is expected as well that some methods
//! only work in case certain feature is paused, this will be useful to implement escape hatches.
//!
//! Features are identified by keys.
//!
//! ## Default implementation:
//!
//! Key "ALL" is understood to pause all "pausable" features at once.
//! Provided implementation is optimized for the case where only a small amount of features are
//! paused at a single moment. If all features are meant to be paused, use "ALL" instead. This is done
//! by storing all paused keys in a single slot on the storage. Notice that unpausing "ALL" will not
//! necessarily unpause all features, if other features are still present in the paused_list.
//!
//! Only owner and self can call `pa_pause_feature` / `pa_unpause_feature`. Requires the contract to
//! be Ownable.
//!
//! ## Credits:
//!
//! Inspired by Open Zeppelin Pausable module:
//! https://github.com/OpenZeppelin/openzeppelin-contracts/blob/master/contracts/security/Pausable.sol
use crate::events::{AsEvent, EventMetadata};
use near_sdk::AccountId;
use serde::Serialize;
use std::collections::HashSet;

pub trait Pausable {
    /// Key of storage slot with list of paused features.
    /// By default b"__PAUSED__" is used.
    fn pa_storage_key(&self) -> Vec<u8>;

    /// Check if a feature is paused
    fn pa_is_paused(&self, key: String) -> bool;

    /// List of all current paused features
    fn pa_all_paused(&self) -> Option<HashSet<String>>;

    /// Pause specified feature.
    fn pa_pause_feature(&mut self, key: String);

    /// Unpause specified feature
    fn pa_unpause_feature(&mut self, key: String);
}

/// Event emitted when a feature is paused.
#[derive(Serialize, Clone)]
pub struct Pause {
    /// Account Id that triggered the pause.
    pub by: AccountId,
    /// Key identifying the feature that was paused.
    pub key: String,
}

impl AsEvent<Pause> for Pause {
    fn metadata(&self) -> EventMetadata<Pause> {
        EventMetadata {
            standard: "Pausable".to_string(),
            version: "1.0.0".to_string(),
            event: "pause".to_string(),
            data: Some(self.clone()),
        }
    }
}

/// Event emitted when a feature is unpaused.
#[derive(Serialize, Clone)]
pub struct Unpause {
    /// Account Id that triggered the unpause.
    pub by: AccountId,
    /// Key identifying the feature that was unpaused.
    pub key: String,
}

impl AsEvent<Unpause> for Unpause {
    fn metadata(&self) -> EventMetadata<Unpause> {
        EventMetadata {
            standard: "Pausable".to_string(),
            version: "1.0.0".to_string(),
            event: "unpause".to_string(),
            data: Some(self.clone()),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests {
    use crate as near_plugins;
    use crate::test_utils::get_context;
    use crate::{if_paused, pause, Ownable, Pausable};
    use std::collections::HashSet;
    use std::convert::TryInto;

    use near_sdk::borsh::BorshDeserialize;
    use near_sdk::borsh::BorshSerialize;
    use near_sdk::{near_bindgen, testing_env, VMContext};

    #[near_bindgen]
    #[derive(Ownable, Pausable)]
    struct Counter {
        counter: u64,
    }

    #[near_bindgen]
    impl Counter {
        /// Specify the owner of the contract in the constructor
        #[init]
        fn new() -> Self {
            let mut contract = Self { counter: 0 };
            contract.owner_set(Some(near_sdk::env::predecessor_account_id()));
            contract
        }

        /// Function can be paused using feature name "increase_1" or "ALL" like:
        /// `contract.pa_pause_feature("increase_1")` or `contract.pa_pause_feature("ALL")`
        ///
        /// If the function is paused, all calls to it will fail. Even calls started from owner or self.
        #[pause]
        fn increase_1(&mut self) {
            self.counter += 1;
        }

        /// Similar to `#[pause]` but use an explicit name for the feature. In this case the feature to be paused
        /// is named "Increase by two". Note that trying to pause it using "increase_2" will not have any effect.
        ///
        /// This can be used to pause a subset of the methods at once without requiring to use "ALL".
        #[pause(name = "Increase by two")]
        fn increase_2(&mut self) {
            self.counter += 2;
        }

        /// Similar to `#[pause]` but owner or self can still call this method. Any subset of {self, owner} can be specified.
        #[pause(except(owner, self))]
        fn increase_4(&mut self) {
            self.counter += 4;
        }

        /// This method can only be called when "increase_1" is paused. Use this macro to create escape hatches when some
        /// features are paused. Note that if "ALL" is specified the "increase_1" is considered to be paused.
        #[if_paused(name = "increase_1")]
        fn decrease_1(&mut self) {
            self.counter -= 1;
        }

        /// Custom use of pause features. Only allow increasing the counter using `careful_increase` if it is below 10.
        fn careful_increase(&mut self) {
            if self.counter >= 10 {
                assert!(
                    !self.pa_is_paused("INCREASE_BIG".to_string()),
                    "Method paused for large values of counter"
                );
            }

            self.counter += 1;
        }
    }

    /// Setup basic account. Owner of the account is `dave.test`
    fn setup_basic() -> (Counter, VMContext) {
        let ctx = get_context();
        testing_env!(ctx.clone());
        let mut counter = Counter::new();
        counter.owner_set(Some("dave.test".to_string().try_into().unwrap()));
        (counter, ctx)
    }

    #[test]
    fn simple() {
        let (mut counter, _) = setup_basic();

        assert_eq!(counter.counter, 0);
        counter.increase_1();
        assert_eq!(counter.counter, 1);
    }

    #[test]
    #[should_panic(expected = r#"Pausable: Method is paused"#)]
    fn test_pause_feature() {
        let (mut counter, mut ctx) = setup_basic();

        ctx.predecessor_account_id = "dave.test".to_string().try_into().unwrap();
        testing_env!(ctx.clone());

        counter.pa_pause_feature("increase_1".to_string());

        ctx.predecessor_account_id = "rick.test".to_string().try_into().unwrap();
        testing_env!(ctx);

        counter.increase_1();
        assert_eq!(counter.counter, 0);
    }

    #[test]
    #[should_panic(expected = r#"Pausable: Method is paused"#)]
    fn test_pause_feature_from_owner() {
        let (mut counter, mut ctx) = setup_basic();

        ctx.predecessor_account_id = "dave.test".to_string().try_into().unwrap();
        testing_env!(ctx.clone());

        counter.pa_pause_feature("increase_1".to_string());

        counter.increase_1();
        assert_eq!(counter.counter, 0);
    }

    #[test]
    #[should_panic(expected = r#"Ownable: Method must be called from owner"#)]
    fn test_pause_only_owner() {
        let (mut counter, mut ctx) = setup_basic();

        ctx.predecessor_account_id = "mallory.test".to_string().try_into().unwrap();
        testing_env!(ctx.clone());

        counter.pa_pause_feature("increase_1".to_string());
    }

    #[test]
    #[should_panic(expected = r#"Ownable: Method must be called from owner"#)]
    fn test_pause_only_owner_not_self() {
        let (mut counter, mut ctx) = setup_basic();

        ctx.predecessor_account_id = "alice.test".to_string().try_into().unwrap();
        testing_env!(ctx.clone());

        counter.pa_pause_feature("increase_1".to_string());
    }

    #[test]
    #[should_panic(expected = r#"Pausable: Method is paused"#)]
    fn test_pause_with_all() {
        let (mut counter, mut ctx) = setup_basic();

        ctx.predecessor_account_id = "dave.test".to_string().try_into().unwrap();
        testing_env!(ctx.clone());

        counter.pa_pause_feature("ALL".to_string());

        ctx.predecessor_account_id = "rick.test".to_string().try_into().unwrap();
        testing_env!(ctx);

        counter.increase_1();
        assert_eq!(counter.counter, 0);
    }

    #[test]
    fn test_not_paused_with_different_key() {
        let (mut counter, mut ctx) = setup_basic();

        ctx.predecessor_account_id = "dave.test".to_string().try_into().unwrap();
        testing_env!(ctx.clone());

        counter.pa_pause_feature("other_feature".to_string());

        ctx.predecessor_account_id = "rick.test".to_string().try_into().unwrap();
        testing_env!(ctx);

        counter.increase_1();
        assert_eq!(counter.counter, 1);
    }

    #[test]
    fn test_work_after_unpause() {
        let (mut counter, mut ctx) = setup_basic();

        ctx.predecessor_account_id = "dave.test".to_string().try_into().unwrap();
        testing_env!(ctx.clone());

        counter.pa_pause_feature("increase_1".to_string());
        counter.pa_unpause_feature("increase_1".to_string());

        ctx.predecessor_account_id = "rick.test".to_string().try_into().unwrap();
        testing_env!(ctx);

        counter.increase_1();
        assert_eq!(counter.counter, 1);
    }

    #[test]
    fn test_paused_list() {
        let (mut counter, mut ctx) = setup_basic();

        ctx.predecessor_account_id = "dave.test".to_string().try_into().unwrap();
        testing_env!(ctx.clone());

        counter.pa_pause_feature("feature_a".to_string());
        assert_eq!(
            counter.pa_all_paused(),
            Some(HashSet::from(["feature_a".to_string()]))
        );

        counter.pa_pause_feature("feature_b".to_string());
        assert_eq!(
            counter.pa_all_paused(),
            Some(HashSet::from([
                "feature_a".to_string(),
                "feature_b".to_string()
            ]))
        );

        counter.pa_unpause_feature("feature_a".to_string());
        assert_eq!(
            counter.pa_all_paused(),
            Some(HashSet::from(["feature_b".to_string()]))
        );
    }

    #[test]
    fn test_is_paused() {
        let (mut counter, mut ctx) = setup_basic();

        ctx.predecessor_account_id = "dave.test".to_string().try_into().unwrap();
        testing_env!(ctx.clone());

        assert_eq!(counter.pa_is_paused("feature_a".to_string()), false);
        counter.pa_pause_feature("feature_a".to_string());
        assert_eq!(counter.pa_is_paused("feature_a".to_string()), true);
        counter.pa_unpause_feature("feature_a".to_string());
        assert_eq!(counter.pa_is_paused("feature_a".to_string()), false);
    }

    #[test]
    fn test_pause_custom_name_ok() {
        let (mut counter, mut ctx) = setup_basic();

        ctx.predecessor_account_id = "dave.test".to_string().try_into().unwrap();
        testing_env!(ctx.clone());

        counter.pa_pause_feature("increase_2".to_string());

        ctx.predecessor_account_id = "rick.test".to_string().try_into().unwrap();
        testing_env!(ctx);

        counter.increase_2();
        assert_eq!(counter.counter, 2);
    }

    #[test]
    #[should_panic(expected = r#"Pausable: Method is paused"#)]
    fn test_pause_custom_name_fail() {
        let (mut counter, mut ctx) = setup_basic();

        ctx.predecessor_account_id = "dave.test".to_string().try_into().unwrap();
        testing_env!(ctx.clone());

        counter.pa_pause_feature("Increase by two".to_string());

        ctx.predecessor_account_id = "rick.test".to_string().try_into().unwrap();
        testing_env!(ctx);

        counter.increase_2();
        assert_eq!(counter.counter, 0);
    }

    #[test]
    fn test_pause_except_self_and_owner() {
        let (mut counter, mut ctx) = setup_basic();

        ctx.predecessor_account_id = "dave.test".to_string().try_into().unwrap();
        testing_env!(ctx.clone());

        counter.pa_pause_feature("increase_4".to_string());

        counter.increase_4();
        assert_eq!(counter.counter, 4);

        ctx.predecessor_account_id = "alice.test".to_string().try_into().unwrap();
        testing_env!(ctx.clone());

        counter.increase_4();
        assert_eq!(counter.counter, 8);
    }

    #[test]
    #[should_panic(expected = r#"Pausable: Method is paused"#)]
    fn test_pause_except_fail() {
        let (mut counter, mut ctx) = setup_basic();

        ctx.predecessor_account_id = "dave.test".to_string().try_into().unwrap();
        testing_env!(ctx.clone());

        counter.pa_pause_feature("increase_4".to_string());

        ctx.predecessor_account_id = "mallory.test".to_string().try_into().unwrap();
        testing_env!(ctx.clone());

        counter.increase_4();
        assert_eq!(counter.counter, 0);
    }

    #[test]
    fn test_custom_big_ok() {
        let (mut counter, mut ctx) = setup_basic();

        ctx.predecessor_account_id = "mallory.test".to_string().try_into().unwrap();
        testing_env!(ctx.clone());

        for _ in 0..20 {
            counter.careful_increase();
        }

        assert_eq!(counter.counter, 20);
    }

    #[test]
    #[should_panic(expected = r#"Method paused for large values of counter"#)]
    fn test_big_fail() {
        let (mut counter, mut ctx) = setup_basic();

        ctx.predecessor_account_id = "dave.test".to_string().try_into().unwrap();
        testing_env!(ctx.clone());

        counter.pa_pause_feature("INCREASE_BIG".to_string());

        ctx.predecessor_account_id = "mallory.test".to_string().try_into().unwrap();
        testing_env!(ctx.clone());

        for _ in 0..20 {
            counter.careful_increase();
        }

        assert_eq!(counter.counter, 20);
    }

    #[test]
    fn test_escape_hatch_ok() {
        let (mut counter, mut ctx) = setup_basic();
        counter.increase_1();
        assert_eq!(counter.counter, 1);
        ctx.predecessor_account_id = "dave.test".to_string().try_into().unwrap();
        testing_env!(ctx.clone());
        counter.pa_pause_feature("increase_1".to_string());
        counter.decrease_1();
        assert_eq!(counter.counter, 0);
    }

    #[test]
    #[should_panic(expected = r#"Pausable: Method must be paused"#)]
    fn test_escape_hatch_fail() {
        let (mut counter, _) = setup_basic();
        counter.increase_1();
        assert_eq!(counter.counter, 1);

        counter.decrease_1();
        assert_eq!(counter.counter, 0);
    }
}
