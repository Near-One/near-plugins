use near_sdk::{env, AccountId};

pub trait Ownable {
    fn owner_storage_key(&self) -> Vec<u8>;

    fn get_owner(&self) -> Option<AccountId>;

    fn set_owner(&mut self, owner: AccountId);

    fn is_owner(&self) -> bool;

    fn is_self(&self) -> bool {
        near_sdk::env::current_account_id() == near_sdk::env::predecessor_account_id()
    }

    fn assert_owner(&self) {
        assert!(self.is_owner(), "Ownable: Function not called from owner");
    }

    fn assert_owner_or_self(&self) {
        assert!(
            self.is_self() || self.is_owner(),
            "Ownable: Function not called from self or owner"
        );
    }
}
