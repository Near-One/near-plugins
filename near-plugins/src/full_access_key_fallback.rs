//! # Full Access Key Fallback
//!
//! Smart contracts can be considered trustless, when there is no Full Access Key (FAK)
//! attached to it. Otherwise owner of the FAK can redeploy or use the funds stored on
//! the smart contract.
//!
//! However some times a FAK is required in order to prevent or fix an unexpected event.
//! This trait allows the contract not to have a FAK, and add one when needed using a
//! custom mechanism.
//!
//! ## Default implementation:
//!
//! Contract must be Ownable. Only the owner can attach a new FAK.
//! The owner can be set to any arbitrary NEAR account id, for example a DAO.
use crate::events::{AsEvent, EventMetadata};
use near_sdk::{AccountId, PublicKey};
use serde::Serialize;

/// Trait describing the functionality of the _Full Access Key Fallback_ plugin.
pub trait FullAccessKeyFallback {
    /// Attach a new full access to the current contract.
    fn attach_full_access_key(&mut self, public_key: PublicKey) -> near_sdk::Promise;
    // fn attach_full_access_key(&mut self, public_key: PublicKey);
}

/// Event emitted every time a new FullAccessKey is added
#[derive(Serialize, Clone)]
pub struct FullAccessKeyAdded {
    /// The account that added the full access key.
    pub by: AccountId,
    /// The public key that was added.
    pub public_key: PublicKey,
}

impl AsEvent<FullAccessKeyAdded> for FullAccessKeyAdded {
    fn metadata(&self) -> EventMetadata<FullAccessKeyAdded> {
        EventMetadata {
            standard: "FullAccessKeyFallback".to_string(),
            version: "1.0.0".to_string(),
            event: "full_access_key_added".to_string(),
            data: Some(self.clone()),
        }
    }
}
