use crate::events::{AsEvent, EventMetadata};
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
use near_sdk::{AccountId, Promise, PublicKey};
use serde::Serialize;

pub trait FullAccessKeyFallback {
    /// Attach a new full access to the current contract.
    fn attach_full_access_key(&mut self, public_key: PublicKey) -> near_sdk::Promise;
    // fn attach_full_access_key(&mut self, public_key: PublicKey);
}

/// Event emitted every time a new FullAccessKey is added
#[derive(Serialize, Clone)]
struct FullAccessKeyAdded {
    by: AccountId,
    public_key: PublicKey,
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

#[cfg(test)]
mod tests {
    // TODO: Make simulation test that verifies key get's added to the account
    use crate::test_utils::get_context;
    use crate::{only, FullAccessKeyFallback, Ownable};
    use near_sdk::{near_bindgen, testing_env, PublicKey};
    use std::convert::TryInto;
    use std::str::FromStr;

    #[near_bindgen]
    #[derive(Ownable, FullAccessKeyFallback)]
    struct Contract;

    fn key() -> PublicKey {
        PublicKey::from_str("ed25519:6E8sCci9badyRkXb3JoRpBj5p8C6Tw41ELDZoiihKEtp").unwrap()
    }

    #[test]
    #[should_panic(expected = r#"Ownable: Method must be called from owner."#)]
    fn not_owner() {
        let ctx = get_context();
        testing_env!(ctx);

        let mut contract = Contract;
        contract.attach_full_access_key(key());
    }

    #[test]
    fn simple() {
        let mut ctx = get_context();
        testing_env!(ctx.clone());

        let mut contract = Contract;
        contract.owner_set(Some("carol.test".to_string().try_into().unwrap()));

        ctx.predecessor_account_id = "carol.test".to_string().try_into().unwrap();
        testing_env!(ctx);

        contract.attach_full_access_key(key());
    }
}
