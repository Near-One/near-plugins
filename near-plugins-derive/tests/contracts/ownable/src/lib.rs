use near_plugins::{only, Ownable};
use near_sdk::borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::{near, AccountId, PanicOnDefault};

#[near(contract_state)]
#[derive(Ownable, PanicOnDefault)]
pub struct Counter {
    counter: u64,
}

#[near]
impl Counter {
    /// Optionally set the owner in the constructor.
    #[init]
    pub fn new(owner: Option<AccountId>) -> Self {
        let mut contract = Self { counter: 0 };
        if owner.is_some() {
            contract.owner_set(owner);
        }
        contract
    }

    /// Returns the value of the counter.
    pub fn get_counter(&self) -> u64 {
        self.counter
    }

    /// Anyone may call this method successfully.
    pub fn increase(&mut self) -> u64 {
        self.counter += 1;
        self.counter
    }

    /// _Only_ the owner or the contract itself may call this method successfully. It panics if
    /// anyone else calls it.
    #[only(self, owner)]
    pub fn increase_2(&mut self) -> u64 {
        self.counter += 2;
        self.counter
    }

    /// _Only_ the owner may call this method successfully. It panics if anyone else calls it.
    #[only(owner)]
    pub fn increase_3(&mut self) -> u64 {
        self.counter += 3;
        self.counter
    }

    /// _Only_ the contract itself may call this method successfully. It panics if anyone else calls
    /// it.
    ///
    /// It is possible to use `#[only(self)]` even if the contract does not derive `Ownable`.
    #[only(self)]
    pub fn increase_4(&mut self) -> u64 {
        self.counter += 4;
        self.counter
    }
}
