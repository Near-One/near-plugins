# Migrating to Separate Pause/Unpause Roles

This guide explains how to migrate your code to use the new `pause_roles` and `unpause_roles` attributes instead of the consolidated `manager_roles` attribute in the Pausable plugin.

## Important Warning About Role Enums

When using `#[derive(AccessControlRole)]`, **never remove existing variants or add new variants in the middle of the enum**. The order of variants is critical because:

1. Each variant is mapped to specific bit positions in the permissions bitflags
2. Removing or reordering variants will cause existing permissions to be mapped incorrectly
3. This can result in accounts unintentionally gaining or losing access to features

**Always add new variants at the end of the enum to preserve existing permission mappings.**

## Changes Required

### Before

Previously, you would define permissions for both pausing and unpausing using a single attribute:

```rust
#[pausable(manager_roles(Role::PauseManager))]
struct Contract {
    // Contract fields
}
```

This meant that any account with the `PauseManager` role could both pause and unpause features.

### After

Now, you need to specify permissions for pausing and unpausing separately:

```rust
#[pausable(
    pause_roles(Role::PauseManager),
    unpause_roles(Role::UnpauseManager)
)]
struct Contract {
    // Contract fields
}
```

With this change, you can:
- Grant an account only the ability to pause features (emergency response)
- Grant a different account only the ability to unpause features (recovery process)
- Grant some accounts both abilities

## Step-by-Step Migration

1. **Update your Role enum** by adding new roles at the end to preserve existing mappings:

   ```rust
   #[derive(AccessControlRole, Deserialize, Serialize, Copy, Clone)]
   #[serde(crate = "near_sdk::serde")]
   pub enum Role {
       // Existing roles (DO NOT change order)
       PauseManager,   // Will now be used for pause permissions only
       // Other existing roles...
       
       // Add new roles at the end only
       UnpauseManager, // New role for unpause permissions
   }
   ```

2. **Update the pausable attribute** to use the new format:

   ```rust
   // Old format
   // #[pausable(manager_roles(Role::PauseManager))]
   
   // New format
   #[pausable(
       pause_roles(Role::PauseManager),
       unpause_roles(Role::UnpauseManager)
   )]
   ```

   To maintain backward compatibility where existing PauseManager accounts can still do both operations:

   ```rust
   #[pausable(
       pause_roles(Role::PauseManager),
       unpause_roles(Role::PauseManager, Role::UnpauseManager)
   )]
   ```

3. **Update contract initialization** to grant the appropriate roles:

   ```rust
   #[init]
   pub fn new(pause_manager: AccountId, unpause_manager: AccountId) -> Self {
       let mut contract = Self { 
           // contract fields 
       };
       
       // Make the contract itself super admin
       contract.acl_init_super_admin(env::current_account_id());
       
       // Grant pause role
       contract.acl_grant_role(Role::PauseManager.into(), pause_manager);
       
       // Grant unpause role (might be the same or different account)
       contract.acl_grant_role(Role::UnpauseManager.into(), unpause_manager);
       
       contract
   }
   ```

4. **Update or add a migration function** if you're upgrading an existing contract:

   ```rust
   #[private]
   pub fn migrate_pause_unpause_roles(&mut self) {
       // Grant UnpauseManager role to existing PauseManager accounts
       // This maintains the same capabilities they had before
       
       let pause_managers = self.acl_get_grantees("PauseManager", 0, 100);
       for account_id in pause_managers {
           self.acl_grant_role(Role::UnpauseManager.into(), account_id);
       }
   }
   ```

5. **Update tests** to test both pause and unpause permissions separately.

## Complete Example

Here's a complete example of a contract using the new separated roles:

```rust
#[access_control(role_type(Role))]
#[near(contract_state)]
#[derive(Pausable, PanicOnDefault)]
#[pausable(
    pause_roles(Role::PauseManager, Role::EmergencyPauser),
    unpause_roles(Role::UnpauseManager, Role::ServiceRestorer)
)]
pub struct Counter {
    counter: u64,
}
```

In this example:
- Accounts with either `PauseManager` or `EmergencyPauser` roles can pause features
- Accounts with either `UnpauseManager` or `ServiceRestorer` roles can unpause features
- An account might have multiple roles (e.g., both pause and unpause capabilities)