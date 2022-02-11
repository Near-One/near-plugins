//! # Ownable:
//!
//! Trait which provides a basic access control mechanism, where
//! there is an account (an owner) that can be granted exclusive access to
//! specific functions.
//!
//! During creation of the contract set the owner using `owner_set`. Protect functions that should
//! only be called by the owner using #[only(owner)].
//!
//! ## Credits:
//!
//! Inspired by Open Zeppelin Ownable module:
//! https://github.com/OpenZeppelin/openzeppelin-contracts/blob/master/contracts/access/Ownable.sol
use crate::events::{AsEvent, EventMetadata};
use near_sdk::AccountId;
use serde::Serialize;

pub trait Ownable {
    /// Key of storage slot to save the current owner.
    /// By default b"__OWNER__" is used.
    fn owner_storage_key(&self) -> Vec<u8>;

    /// Return the current owner of the contract. Result must be a NEAR valid account id
    /// or None, in case the account doesn't have an owner.
    fn owner_get(&self) -> Option<AccountId>;

    /// Replace the current owner of the contract by a new owner. Triggers an event of type
    /// OwnershipTransferred. Use `None` to remove the owner of the contract all together.
    ///
    /// # Default Implementation:
    ///
    /// Only the current owner can call this method. If no owner is set, only self can call this
    /// method. Notice that if the owner is set, self will not be able to call `owner_set` by default.
    fn owner_set(&mut self, owner: Option<AccountId>);

    /// Return true if the predecessor account id is the owner of the contract.
    fn owner_is(&self) -> bool;
}

/// Event emitted when ownership is changed.
#[derive(Serialize, Clone)]
pub struct OwnershipTransferred {
    pub previous_owner: Option<AccountId>,
    pub new_owner: Option<AccountId>,
}

impl AsEvent<OwnershipTransferred> for OwnershipTransferred {
    fn metadata(&self) -> EventMetadata<OwnershipTransferred> {
        EventMetadata {
            standard: "Ownable".to_string(),
            version: "1.0.0".to_string(),
            event: "ownership_transferred".to_string(),
            data: Some(self.clone()),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate as near_plugins;
    use crate::test_utils::get_context;
    use crate::{only, Ownable};
    use near_sdk::{near_bindgen, testing_env, VMContext};
    use std::convert::TryInto;

    #[near_bindgen]
    #[derive(Ownable)]
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

        /// Only owner account, or the contract itself can call this method.
        #[only(self, owner)]
        fn protected(&mut self) {
            self.counter += 1;
        }

        /// *Only* owner account can call this method.
        #[only(owner)]
        fn protected_owner(&mut self) {
            self.counter += 1;
        }

        /// *Only* self account can call this method. This can be used even if the contract is not Ownable.
        #[only(self)]
        fn protected_self(&mut self) {
            self.counter += 1;
        }

        /// Everyone can call this method
        fn unprotected(&mut self) {
            self.counter += 1;
        }
    }

    /// Setup basic account. Owner of the account is `carol.test`
    fn setup_basic() -> (Counter, VMContext) {
        let ctx = get_context();
        testing_env!(ctx.clone());
        let mut counter = Counter::new();
        counter.owner_set(Some("carol.test".to_string().try_into().unwrap()));
        (counter, ctx)
    }

    #[test]
    fn build_contract() {
        let _ = setup_basic();
    }

    #[test]
    fn test_is_owner() {
        let (counter, mut ctx) = setup_basic();
        assert!(!counter.owner_is());
        ctx.predecessor_account_id = "carol.test".to_string().try_into().unwrap();
        testing_env!(ctx);
        assert!(counter.owner_is());
    }

    #[test]
    fn test_set_owner_ok() {
        let (mut counter, mut ctx) = setup_basic();
        ctx.predecessor_account_id = "carol.test".to_string().try_into().unwrap();
        testing_env!(ctx);
        counter.owner_set(Some("eve.test".to_string().try_into().unwrap()));
    }

    #[test]
    #[should_panic(expected = r#"Ownable: Only owner can update current owner"#)]
    fn test_set_owner_fail() {
        let (mut counter, _) = setup_basic();
        counter.owner_set(Some("eve.test".to_string().try_into().unwrap()));
    }

    #[test]
    fn test_remove_owner() {
        let (mut counter, mut ctx) = setup_basic();
        ctx.predecessor_account_id = "carol.test".to_string().try_into().unwrap();
        testing_env!(ctx);
        counter.owner_set(None);
        assert_eq!(counter.owner_get(), None);
    }

    #[test]
    fn counter_unprotected() {
        let (mut counter, _) = setup_basic();
        assert_eq!(counter.counter, 0);
        counter.unprotected();
        assert_eq!(counter.counter, 1);
    }

    #[test]
    fn protected_self_ok() {
        let (mut counter, _) = setup_basic();

        counter.protected_self();
        assert_eq!(counter.counter, 1);
    }

    #[test]
    #[should_panic(expected = r#"Method is private"#)]
    fn protected_self_fail() {
        let (mut counter, mut ctx) = setup_basic();

        ctx.predecessor_account_id = "mallory.test".to_string().try_into().unwrap();
        testing_env!(ctx);

        counter.protected_self();
        assert_eq!(counter.counter, 0);
    }

    #[test]
    #[should_panic(expected = r#"Method is private"#)]
    fn protected_self_owner_fail() {
        let (mut counter, mut ctx) = setup_basic();

        ctx.predecessor_account_id = "carol.test".to_string().try_into().unwrap();
        testing_env!(ctx);

        counter.protected_self();
        assert_eq!(counter.counter, 0);
    }

    #[test]
    fn protected_owner_ok() {
        let (mut counter, mut ctx) = setup_basic();

        ctx.predecessor_account_id = "carol.test".to_string().try_into().unwrap();
        testing_env!(ctx);

        counter.protected_owner();
        assert_eq!(counter.counter, 1);
    }

    #[test]
    #[should_panic(expected = r#"Ownable: Method must be called from owner"#)]
    fn protected_owner_self_fail() {
        let (mut counter, _) = setup_basic();

        counter.protected_owner();
        assert_eq!(counter.counter, 0);
    }

    #[test]
    #[should_panic(expected = r#"Ownable: Method must be called from owner"#)]
    fn protected_owner_fail() {
        let (mut counter, mut ctx) = setup_basic();

        ctx.predecessor_account_id = "mallory.test".to_string().try_into().unwrap();
        testing_env!(ctx);

        counter.protected_owner();
        assert_eq!(counter.counter, 0);
    }

    #[test]
    fn protected_ok() {
        let (mut counter, mut ctx) = setup_basic();

        counter.protected();
        assert_eq!(counter.counter, 1);

        ctx.predecessor_account_id = "carol.test".to_string().try_into().unwrap();
        testing_env!(ctx);

        counter.protected();
        assert_eq!(counter.counter, 2);
    }

    #[test]
    #[should_panic(expected = r#"Method is private"#)]
    fn protected_fail() {
        let (mut counter, mut ctx) = setup_basic();

        ctx.predecessor_account_id = "mallory.test".to_string().try_into().unwrap();
        testing_env!(ctx);

        counter.protected();
        assert_eq!(counter.counter, 0);
    }
}
