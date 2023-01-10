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

/// Trait describing the functionality of the _Ownable_ plugin.
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
    ///
    /// # View calls
    ///
    /// This method fails in view calls since getting the predecessor account id is [not allowed] in
    /// view calls. A workaround is using [`Self::owner_get`] and checking the returned account id.
    ///
    /// [not allowed]: https://nomicon.io/Proposals/view-change-method
    fn owner_is(&self) -> bool;
}

/// Event emitted when ownership is changed.
#[derive(Serialize, Clone)]
pub struct OwnershipTransferred {
    /// The previous owner, if any.
    pub previous_owner: Option<AccountId>,
    /// The new owner, if any.
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
