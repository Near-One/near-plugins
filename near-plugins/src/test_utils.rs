use near_sdk::VMContext;
use std::convert::TryInto;

pub fn get_context() -> VMContext {
    VMContext {
        current_account_id: "alice.test".to_string().try_into().unwrap(),
        signer_account_id: "bob.test".to_string().try_into().unwrap(),
        signer_account_pk: vec![0, 1, 2],
        predecessor_account_id: "alice.test".to_string().try_into().unwrap(),
        input: vec![],
        block_index: 0,
        block_timestamp: 0,
        account_balance: 0,
        account_locked_balance: 0,
        storage_usage: 10_000,
        attached_deposit: 0,
        prepaid_gas: 10u64.pow(18),
        random_seed: vec![0, 1, 2],
        output_data_receivers: vec![],
        epoch_height: 19,
        view_config: None,
    }
}
