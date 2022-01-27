/// # Full Access Key Fallback
///
/// Smart contracts can be considered trustless, when there is no Full Access Key (FAK)
/// attached to it. Otherwise owner of the FAK can redeploy or use the funds stored on
/// the smart contract.
///
/// However some times a FAK is required in order to prevent or fix an unexpected event.
/// This trait allows the contract not to have a FAK, and add one when needed using a
/// custom mechanism.
///
/// ## Default implementation:
///
/// Contract must be Ownable. Only the owner can attach a new FAK.
/// The owner can be set to any arbitrary NEAR account id, for example a DAO.
use near_sdk::{AccountId, PublicKey};

pub trait FullAccessKeyFallback {
    /// Attach a new full access to the current contract.
    fn attach_full_access_key(&mut self, public_key: PublicKey) -> near_sdk::Promise;
}

/// Event emitted every time a new FullAccessKey is added
struct FullAccessKeyAdded {
    by: AccountId,
    // TODO: Serialize key as base58 on the event side
    public_key: PublicKey,
}
