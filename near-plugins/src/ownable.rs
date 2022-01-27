/// # Ownable
///
/// Trait which provides a basic access control mechanism, where
/// there is an account (an owner) that can be granted exclusive access to
/// specific functions.
///
/// During creation of the contract set the owner using `owner_set`. Protect functions that should
/// only be called by the owner using #[only(owner)].
///
/// ## Credits:
///
/// Inspired by Open Zeppelin Ownable module:
/// https://github.com/OpenZeppelin/openzeppelin-contracts/blob/master/contracts/access/Ownable.sol
use near_sdk::{env, AccountId};

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
struct OwnershipTransferred {
    previous_owner: Option<AccountId>,
    new_owner: Option<AccountId>,
}
