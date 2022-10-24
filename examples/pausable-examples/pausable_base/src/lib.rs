use near_plugins::Ownable;
use near_plugins::Pausable;
use near_sdk::near_bindgen;
use near_plugins_derive::{pause, if_paused};
use borsh::{BorshSerialize, BorshDeserialize};

#[near_bindgen]
#[derive(Ownable, Pausable, Default, BorshSerialize, BorshDeserialize)]
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

    /// Function can be paused using feature name "increase_1" or "ALL" like:
    /// `contract.pa_pause_feature("increase_1")` or `contract.pa_pause_feature("ALL")`
    ///
    /// If the function is paused, all calls to it will fail. Even calls started from owner or self.
    #[pause]
    pub fn increase_1(&mut self) {
        self.counter += 1;
    }

    /// Similar to `#[pause]` but use an explicit name for the feature. In this case the feature to be paused
    /// is named "Increase by two". Note that trying to pause it using "increase_2" will not have any effect.
    ///
    /// This can be used to pause a subset of the methods at once without requiring to use "ALL".
    #[pause(name = "Increase by two")]
    pub fn increase_2(&mut self) {
        self.counter += 2;
    }

    /// Similar to `#[pause]` but owner or self can still call this method. Any subset of {self, owner} can be specified.
    #[pause(except(owner, self))]
    pub fn increase_4(&mut self) {
        self.counter += 4;
    }

    /// This method can only be called when "increase_1" is paused. Use this macro to create escape hatches when some
    /// features are paused. Note that if "ALL" is specified the "increase_1" is considered to be paused.
    #[if_paused(name = "increase_1")]
    pub fn decrease_1(&mut self) {
        self.counter -= 1;
    }

    /// Custom use of pause features. Only allow increasing the counter using `careful_increase` if it is below 10.
    pub fn careful_increase(&mut self) {
        if self.counter >= 10 {
            assert!(
                !self.pa_is_paused("INCREASE_BIG".to_string()),
                "Method paused for large values of counter"
            );
        }

        self.counter += 1;
    }

    pub fn get_counter(&self) -> u64 {
        self.counter
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use near_sdk::AccountId;
    use near_plugins_test_utils::*;

    const WASM_FILEPATH: &str = "../../target/wasm32-unknown-unknown/release/pausable_base.wasm";

    #[tokio::test]
    async fn base_scenario() {
        let (contract_holder, contract) = get_contract(WASM_FILEPATH).await;

        assert!(call!(contract,"new").await);

        let next_owner = get_subaccount(&contract_holder, "next_owner").await;
        assert!(call!(contract, "owner_set", &json!({"owner": next_owner.id()})).await);
        let current_owner: Option::<AccountId> = view!(contract, "owner_get");
        assert_eq!(current_owner.unwrap().as_str(), next_owner.id().as_str());

        let alice = get_subaccount(&contract_holder, "alice").await;

        assert!(call!(&alice, contract, "increase_1").await);
        check_counter(&contract, 1).await;

        assert!(!call!(&alice, contract, "pa_pause_feature", &json!({"key": "increase_1"})).await);
        assert!(call!(&next_owner, contract, "pa_pause_feature", &json!({"key": "increase_1"})).await);

        assert!(!call!(&alice, contract, "increase_1").await);
        check_counter(&contract, 1).await;

        assert!(!call!(&next_owner, contract, "increase_1").await);
        check_counter(&contract, 1).await;

        assert!(!call!(&contract_holder, contract, "pa_unpause_feature", &json!({"key": "increase_1"})).await);
        assert!(call!(&next_owner, contract, "pa_unpause_feature", &json!({"key": "increase_1"})).await);

        assert!(call!(&alice, contract, "increase_1").await);

        check_counter(&contract, 2).await;
    }
}
