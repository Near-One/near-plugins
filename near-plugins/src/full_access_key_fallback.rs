/// Smart contracts can be considered trustless, when there is no Full Access Key (FAK)
/// attached to it. Otherwise owner of the FAK can redeploy or use the funds stored on
/// the smart contract.
///
/// However some times a FAK is required in order to prevent or fix an unexpected event.
/// This trait allows the contract not to have a FAK, and add one when needed using a
/// custom mechanism.
///
/// Default implementation derived let the owner add a FAK. Contract must be Ownable.
/// The owner can be set to any arbitrary NEAR account id, for example a DAO.
use near_sdk::PublicKey;

pub trait FullAccessKeyFallback {
    fn add_full_access_key(&mut self, public_key: PublicKey) -> near_sdk::Promise;
}
