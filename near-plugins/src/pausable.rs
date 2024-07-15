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
//! Provided implementation is optimized for the case where only a small amount of features is
//! paused at a single moment. If all features should be paused, use "ALL" instead. This is done
//! by storing all paused keys in a single slot on the storage. Notice that unpausing "ALL" will not
//! necessarily unpause all features, if other features are still present in the `paused_list`.
//!
//! As a precondition for being `Pausable` a contract must be `AccessControllable`. Access control
//! is used to define the permissions required to pause and unpause features. In addition, grantees
//! of access control roles may be allowed to call methods that are `#[pause]` or `#[if_paused]`
//! unrestrictedly via the `except` argument.
//!
//! ## Credits:
//!
//! Inspired by Open Zeppelin Pausable module:
//! `https://github.com/OpenZeppelin/openzeppelin-contracts/blob/master/contracts/security/Pausable.sol`
use crate::events::{AsEvent, EventMetadata};
use near_sdk::AccountId;
use serde::Serialize;
use std::collections::HashSet;

/// Trait describing the functionality of the `Pausable` plugin.
pub trait Pausable {
    /// Returns the key of the storage slot which contains the list of features that are paused. By
    /// default `b"__PAUSED__"` is used.
    ///
    /// Attribute `paused_storage_key` can be used to set a different key:
    ///
    /// ```ignore
    /// #[pausable(paused_storage_key="CUSTOM_KEY")]
    /// struct Contract { /* ... */}
    /// ```
    fn pa_storage_key(&self) -> &'static [u8];

    /// Returns whether feature `key` is paused.
    fn pa_is_paused(&self, key: String) -> bool;

    /// Returns all features that are currently paused.
    fn pa_all_paused(&self) -> Option<HashSet<String>>;

    /// Pauses feature `key`. This method fails if the caller has not been granted one of the access
    /// control `manager_roles` passed to the `Pausable` plugin.
    ///
    /// It returns `true` if the feature is paused as a result of this function call and `false` if
    /// the feature was already paused. In either case, the feature is paused after the function
    /// returns successfully.
    ///
    /// If the feature is newly paused (the return value is `true`), the following event will be
    /// emitted:
    ///
    /// ```json
    /// {
    ///   "standard":"Pausable",
    ///   "version":"1.0.0",
    ///   "event":"pause",
    ///   "data":
    ///     {
    ///       "by":"<OWNER_ACCOUNT>",
    ///       "key":"<KEY>"
    ///     }
    /// }
    /// ```
    fn pa_pause_feature(&mut self, key: String) -> bool;

    /// Unpauses feature `key`. This method fails if the caller has not been granted one of the
    /// access control `manager_roles` passed to the `Pausable` plugin.
    ///
    /// It returns whether the feature was paused, i.e. `true` if the feature was paused and
    /// otherwise `false`. In either case, the feature is unpaused after the function returns
    /// successfully.
    ///
    /// If the feature was paused (the return value is `true`), the following event will be emitted:
    ///
    /// ```json
    /// {
    ///    "standard":"Pausable",
    ///    "version":"1.0.0",
    ///    "event":"unpause",
    ///    "data":
    ///    {
    ///       "by":"<OWNER_ACCOUNT>",
    ///       "key":"<KEY>"
    ///    }
    /// }
    /// ```
    fn pa_unpause_feature(&mut self, key: String) -> bool;
}

/// Event emitted when a feature is paused.
#[derive(Serialize, Clone)]
pub struct Pause {
    /// Account Id that triggered the pause.
    pub by: AccountId,
    /// Key identifying the feature that was paused.
    pub key: String,
}

impl AsEvent<Self> for Pause {
    fn metadata(&self) -> EventMetadata<Self> {
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

impl AsEvent<Self> for Unpause {
    fn metadata(&self) -> EventMetadata<Self> {
        EventMetadata {
            standard: "Pausable".to_string(),
            version: "1.0.0".to_string(),
            event: "unpause".to_string(),
            data: Some(self.clone()),
        }
    }
}
