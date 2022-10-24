use near_plugins::Ownable;
use near_sdk::near_bindgen;
use near_plugins_derive::only;
use borsh::{BorshSerialize, BorshDeserialize};

#[near_bindgen]
#[derive(Ownable, Default, BorshSerialize, BorshDeserialize)]
struct Counter {
    counter: u64,
}

#[near_bindgen]
impl Counter {
    /// Specify the owner of the contract in the constructor
    #[init]
    pub fn new() -> Self {
        let mut contract = Self { counter: 0 };
        contract.owner_set(Some(near_sdk::env::predecessor_account_id()));
        contract
    }

    /// Only owner account, or the contract itself can call this method.
    #[only(self, owner)]
    pub fn protected(&mut self) {
        self.counter += 1;
    }

    /// *Only* owner account can call this method.
    #[only(owner)]
    pub fn protected_owner(&mut self) {
        self.counter += 1;
    }

    /// *Only* self account can call this method. This can be used even if the contract is not Ownable.
    #[only(self)]
    pub fn protected_self(&mut self) {
        self.counter += 1;
    }

    /// Everyone can call this method
    pub fn unprotected(&mut self) {
        self.counter += 1;
    }

    /// View method returns the value of the counter. Everyone can call it
    pub fn get_counter(&self) -> u64 {
        self.counter
    }
}


#[cfg(test)]
mod tests {
    use serde_json::json;
    use near_sdk::AccountId;
    use near_plugins_test_utils::*;

    const WASM_FILEPATH: &str = "../../target/wasm32-unknown-unknown/release/ownable_base.wasm";

    #[tokio::test]
    async fn base_scenario() {
        let (contract_holder, contract) = get_contract(WASM_FILEPATH).await;

        assert!(call!(contract, "new").await);

        let current_owner: Option::<AccountId> = view!(contract, "owner_get");
        assert_eq!(current_owner.unwrap().as_str(), contract_holder.id().as_str());

        check_counter(&contract, 0).await;

        assert!(call!(contract, "protected").await);
        assert!(call!(contract, "protected_owner").await);
        assert!(call!(contract, "protected_self").await);
        assert!(call!(contract, "unprotected").await);

        check_counter(&contract, 4).await;

        let next_owner = get_subaccount(&contract_holder, "next_owner").await;
        assert!(!call!(&next_owner, contract, "protected").await);
        assert!(!call!(&next_owner, contract, "protected_owner").await);
        assert!(!call!(&next_owner, contract, "protected_self").await);
        assert!(call!(&next_owner, contract, "unprotected").await);

        check_counter(&contract, 5).await;

        assert!(call_arg(&contract, "owner_set", &json!({"owner": next_owner.id()})).await);

        let current_owner: Option::<AccountId> = view!(contract, "owner_get");
        assert_ne!(current_owner.clone().unwrap().as_str(), contract_holder.id().as_str());
        assert_eq!(current_owner.unwrap().as_str(), next_owner.id().as_str());

        assert!(call!(&next_owner, contract, "protected").await);
        assert!(call!(&next_owner, contract, "protected_owner").await);
        assert!(!call!(&next_owner, contract, "protected_self").await);
        assert!(call!(&next_owner, contract, "unprotected").await);

        check_counter(&contract, 8).await;

        assert!(call!(contract, "protected").await);
        assert!(!call!(contract, "protected_owner").await);
        assert!(call!(contract, "protected_self").await);
        assert!(call!(contract, "unprotected").await);

        check_counter(&contract, 11);
    }

    #[tokio::test]
    async fn null_owner() {
        let (_, contract) = get_contract(WASM_FILEPATH).await;
        assert!(call!(contract,"new").await);

        assert!(call!(contract, "owner_set", &json!({"owner": Option::<AccountId>::None})).await);

        let current_owner: Option::<AccountId> = view!(contract, "owner_get");
        assert_eq!(current_owner, None);

        assert!(call!(contract, "protected").await);
        assert!(!call!(contract, "protected_owner").await);
        assert!(call!(contract, "protected_self").await);
        assert!(call!(contract, "unprotected").await);

        check_counter(&contract, 3).await;

        assert!(call!(contract, "owner_set", &json!({"owner": contract.id().as_str()})).await);
        assert!(call!(contract, "protected").await);
        assert!(call!(contract, "protected_owner").await);
        assert!(call!(contract, "protected_self").await);
        assert!(call!(contract, "unprotected").await);

        check_counter(&contract, 7).await;
    }

    #[tokio::test]
    async fn check_owner_storage_key() {
        let (_, contract) = get_contract(WASM_FILEPATH).await;
        assert!(call!(contract,"new").await);

        let owner_storage_key: Vec<u8> = view!(contract, "owner_storage_key");
        assert_eq!(owner_storage_key, "__OWNER__".as_bytes().to_vec());
    }
}
