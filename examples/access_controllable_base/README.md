# Example of using Access Control plugin

An access control mechanism that allows you to specify which groups of users can have access to certain functions.

```rust
use near_plugins::AccessControllable;
use near_plugins::AccessControlRole;
use near_plugins_derive::access_control;
use near_plugins_derive::access_control_any;
use near_sdk::near_bindgen;
use borsh::{BorshSerialize, BorshDeserialize};
use near_plugins::events::AsEvent;
use near_sdk::env;

/// All types of access groups
#[derive(AccessControlRole, Clone, Copy)]
pub enum UsersGroups {
    GroupA,
    GroupB,
}

#[near_bindgen]
#[access_control(role_type="UsersGroups")]
#[derive(Default, BorshSerialize, BorshDeserialize)]
struct Counter {
    counter: u64,
}

#[near_bindgen]
impl Counter {
    /// In the constructor we set up a super admin, 
    /// which can control the member lists of all user groups
    #[init]
    pub fn new() -> Self {
        let mut contract: Counter = Self{
            counter: 0,
            __acl: __Acl::default(),
        };

        contract.__acl.init_super_admin(&near_sdk::env::predecessor_account_id());

        contract
    }

    /// unprotected function, every one can call this function
    pub fn unprotected(&mut self) {
        self.counter += 1;
    }

    /// only the users from GroupA can call this method
    #[access_control_any(roles(UsersGroups::GroupA))]
    pub fn level_a_incr(&mut self) {
        self.counter += 1;
    }

    /// only the users from GroupA or GroupB can call this method
    #[access_control_any(roles(UsersGroups::GroupA, UsersGroups::GroupB))]
    pub fn level_ab_incr(&mut self) {
        self.counter += 1;
    }


    /// view method for get current counter value, every one can use it
    pub fn get_counter(&self) -> u64 {
        self.counter
    }
}
```

