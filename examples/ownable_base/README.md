# Example using Ownable plugin

Basic access control mechanism that allows only an authorized account id to call certain methods. Note this account id can belong either to a regular user, or it could be a contract (a DAO for example).

```Rust
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
```

## Preparation steps for demonstration

## Test running instruction
