use near_sdk::{Gas, VMContext};
use std::convert::TryInto;

#[allow(dead_code)]
pub(crate) fn get_context() -> VMContext {
    VMContext {
        current_account_id: "alice.test".to_string().try_into().unwrap(),
        signer_account_id: "bob.test".to_string().try_into().unwrap(),
        signer_account_pk: "ed25519:6E8sCci9badyRkXb3JoRpBj5p8C6Tw41ELDZoiihKEtp"
            .parse()
            .unwrap(),
        predecessor_account_id: "alice.test".to_string().try_into().unwrap(),
        input: vec![],
        block_index: 0,
        block_timestamp: 0,
        account_balance: 0,
        account_locked_balance: 0,
        storage_usage: 10_000,
        attached_deposit: 0,
        prepaid_gas: Gas(10u64.pow(18)),
        random_seed: [0; 32],
        output_data_receivers: vec![],
        epoch_height: 19,
        view_config: None,
    }
}
