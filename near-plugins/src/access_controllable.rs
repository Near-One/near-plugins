//! # `AccessControllable`
//!
//! A trait specifying an interface to manage permissions via roles and access control lists. A
//! contract that is `AccessControllable` may restrict functions or features to accounts that have
//! been granted permissions.
//!
//! ## Roles
//!
//! Permissions are based on roles defined by smart contract developers. In the default
//! implementation provided by `near-plugins`, roles are represented by enum variants.
//!
//! # Controlling access
//!
//! Using the `#[access_control_any(roles(...))]` macro on a contract method restricts access to the
//! method to grantees of the specified `roles`. The method panics if it is called by an account
//! which is not a grantee any of the `roles`.
//!
//! In addition, methods like `AccessControllable::has_role` can be used within other contract
//! methods to restrict access to certain features or actions.
//!
//! ## Granting and revoking permissions
//!
//! Admins can grant roles to and revoke them from accounts. Each role has its own set of admins,
//! which may contain zero or multiple admin accounts. An admin is allowed to add and remove other
//! admin accounts. Note that admin permissions differ from role permissions: an account which is
//! admin for role `r` but not a grantee of role `r` may not use methods or features restricted to
//! role `r`.
//!
//! Besides (regular) admins the `AccessControllable` trait also defines super-admins. A super-admin
//! is considered admin for every role. An `AccessControllable` contract can have zero or more
//! super-admins.
//!
//! ## Credits
//!
//! Inspired by `OpenZeppelin`'s
//! [AccessControl](https://docs.openzeppelin.com/contracts/3.x/api/access#AccessControl) module.

use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::AccountId;
use std::collections::HashMap;

/// # Representation of roles
///
/// This trait is unaware of the concrete type used to represent roles. It is
/// not possible to use a generic type `R` since `near-sdk` [does not support]
/// `impl` type parameters.
///
/// ```ignore
/// // This is not possible:
/// impl<R> AccessControllable<R> for Contract {/* ... */}
/// ```
///
/// Instead, roles are represented by `u8`, which allows contract developers to
/// define their own enum whose variants are converted to `u8`.
///
/// [does not support]: https://github.com/near/near-sdk-rs/blob/9d99077c6acfde68c06845f2a1eb2b5ed7983401/near-sdk/compilation_tests/impl_generic.stderr
pub trait AccessControllable {
    /// Returns the storage prefix for collections related to access control.
    /// `b"__acl"` is used by default.
    ///
    /// Attribute `storage_prefix` can be used to set a different prefix:
    ///
    /// ```ignore
    /// #[access_controllable(storage_prefix="CUSTOM_KEY")]
    /// struct Contract { /* ... */}
    /// ```
    fn acl_storage_prefix() -> &'static [u8];

    /// Returns the names of all variants of the enum that represents roles.
    ///
    /// In the default implementation provided by this crate, this enum is defined by contract
    /// developers using the plugin and passed as an attribute to the `access_controllable` macro.
    ///
    /// A vector containing _all_ variant names is returned since the default implementation limits
    /// the number of variants to [`near_plugins_derive::access_control_role::MAX_ROLE_VARIANTS`].
    /// This allows for a simpler user experience compared to the iterator based approach of
    /// [`Self::acl_get_admins`], for example. For custom implementations of this it is advised to
    /// limit the number of role variants as well.
    ///
    /// Event though it might not be used, this method takes parameter `&self` to be [available in
    /// view calls].
    ///
    /// [available in view calls]: https://stackoverflow.com/q/66715815
    fn acl_role_variants(&self) -> Vec<&'static str>;

    /// Adds `account_id` as super-admin __without__ checking any permissions in
    /// case there are no super-admins. If there is already a super-admin, it
    /// has no effect. This function can be used to add a super-admin during
    /// contract initialization. Moreover, it may provide a recovery mechanism
    /// if (mistakenly) all super-admins have been removed.
    ///
    /// The return value indicates whether `account_id` was added as
    /// super-admin.
    ///
    /// It is `#[private]` in the implementation provided by this trait, i.e.
    /// only the contract itself may call this method.
    ///
    /// Despite the restrictions of this method, it is possible to add multiple
    /// super-admins using [`acl_add_super_admin`].
    ///
    /// If a super-admin is added, the following event will be emitted:
    ///
    /// ```json
    /// {
    ///    "standard":"AccessControllable",
    ///    "version":"1.0.0",
    ///    "event":"super_admin_added",
    ///    "data":{
    ///       "account":"<SUPER_ADMIN_ACCOUNT>",
    ///       "by":"<CONTRACT_ACCOUNT>"
    ///    }
    /// }
    /// ```
    fn acl_init_super_admin(&mut self, account_id: AccountId) -> bool;

    /// Adds `account_id` as super-admin provided that the predecessor has sufficient permissions,
    /// i.e. is a super-admin as defined by [`acl_is_super_admin`]. To add the first super-admin,
    /// [`acl_init_super_admin`] can be used.
    ///
    /// In case of sufficient permissions, the returned `Some(bool)` indicates whether `account_id`
    /// is a new super-admin. Without permissions, `None` is returned and internal state is not
    /// modified.
    ///
    /// Note that there may be multiple (or zero) super-admins.
    ///
    /// If a super-admin is added, the following event will be emitted:
    ///
    /// ```json
    /// {
    ///    "standard":"AccessControllable",
    ///    "version":"1.0.0",
    ///    "event":"super_admin_added",
    ///    "data":{
    ///       "account":"<NEW_SUPER_ADMIN>",
    ///       "by":"<SUPER_ADMIN>"
    ///    }
    /// }
    /// ```
    fn acl_add_super_admin(&mut self, account_id: AccountId) -> Option<bool>;

    /// Returns whether `account_id` is a super-admin. A super-admin has admin
    /// permissions for every role. However, a super-admin is not considered
    /// grantee of any role.
    fn acl_is_super_admin(&self, account_id: AccountId) -> bool;

    /// Revoke super-admin permissions from `account_id` provided that the
    /// predecessor has sufficient permissions, i.e. is a super-admin as defined
    /// by [`acl_is_super_admin`]. This means a super-admin may revoke
    /// super-admin permissions from any other super-admin.
    ///
    /// In case of sufficient permissions, the returned `Some(bool)` indicates
    /// whether `account_id` was a super-admin. Without permissions, `None` is
    /// returned and internal state is not modified.
    ///
    /// If super-admin permissions are revoked, the following event will be
    /// emitted:
    ///
    /// ```json
    /// {
    ///    "standard":"AccessControllable",
    ///    "version":"1.0.0",
    ///    "event":"super_admin_revoked",
    ///    "data":{
    ///       "account":"<PREVIOUSLY_SUPER_ADMIN>",
    ///       "by":"<SUPER_ADMIN>"
    ///    }
    /// }
    /// ```
    fn acl_revoke_super_admin(&mut self, account_id: AccountId) -> Option<bool>;

    /// Transfer super-admin permissions from the predecessor to `account_id` provided that the
    /// predecessor has sufficient permissions, i.e. is a super-admin as defined
    /// by [`acl_is_super_admin`]. This function allows a super-admin to revoke the permission from
    /// themselves and add `account_id` as super-admin. While it is a helper for use cases which
    /// require this transfer, it should be noted that `AccessControllable` allows having more than
    /// one super-admin.
    ///
    /// In case of sufficient permissions, the returned `Some(bool)` indicates
    /// whether `account_id` is a new super-admin. Without permissions, `None` is
    /// returned and internal state is not modified.
    ///
    /// If super-admin permissions are transferred, the following events will be
    /// emitted:
    ///
    /// ```json
    /// {
    ///    "standard":"AccessControllable",
    ///    "version":"1.0.0",
    ///    "event":"super_admin_revoked",
    ///    "data":{
    ///       "account":"<PREVIOUSLY_SUPER_ADMIN>",
    ///       "by":"<SUPER_ADMIN>"
    ///    }
    /// }
    /// ```
    ///
    /// ```json
    /// {
    ///    "standard":"AccessControllable",
    ///    "version":"1.0.0",
    ///    "event":"super_admin_added",
    ///    "data":{
    ///       "account":"<SUPER_ADMIN_ACCOUNT>",
    ///       "by":"<CONTRACT_ACCOUNT>"
    ///    }
    /// }
    /// ```
    fn acl_transfer_super_admin(&mut self, account_id: AccountId) -> Option<bool>;

    /// Makes `account_id` an admin provided that the predecessor has sufficient
    /// permissions, i.e. is an admin as defined by [`acl_is_admin`].
    ///
    /// In case of sufficient permissions, the returned `Some(bool)` indicates
    /// whether `account_id` is a new admin for `role`. Without permissions,
    /// `None` is returned and internal state is not modified.
    ///
    /// Note that any role may have multiple (or zero) admins.
    ///
    /// If an admin is added, the following event will be emitted:
    ///
    /// ```json
    /// {
    ///    "standard":"AccessControllable",
    ///    "version":"1.0.0",
    ///    "event":"admin_added",
    ///    "data": {
    ///       "role":"<ROLE>",
    ///       "account":"<NEW_ADMIN>",
    ///       "by":"<ADMIN>"
    ///    }
    /// }
    /// ```
    fn acl_add_admin(&mut self, role: String, account_id: AccountId) -> Option<bool>;

    /// Returns whether `account_id` is an admin for `role`. Super-admins are
    /// admins for _every_ role.
    ///
    /// Note that adding an account as admin for `role` does not make the
    /// account a grantee of `role`. Instead, `role` has to be granted
    /// explicitly. The same applies to super-admins.
    fn acl_is_admin(&self, role: String, account_id: AccountId) -> bool;

    /// Revokes admin permissions for `role` from `account_id` provided that the
    /// predecessor has sufficient permissions, i.e. is an admin as defined by
    /// [`acl_is_admin`]. This means an admin for `role` may revoke admin
    /// permissions from any other account that is admin for `role`.
    ///
    /// In case of sufficient permissions, the returned `Some(bool)` indicates
    /// whether `account_id` was an admin for `role`. Without permissions,
    /// `None` is returned and internal state is not modified.
    ///
    /// If an admin is revoked, the following event will be emitted:
    ///
    /// ```json
    /// {
    ///    "standard":"AccessControllable",
    ///    "version":"1.0.0",
    ///    "event":"admin_revoked",
    ///    "data":{
    ///       "role":"<ROLE>",
    ///       "account":"<PREVIOUSLY_ADMIN>",
    ///       "by":"<ADMIN>"
    ///    }
    /// }
    /// ```
    fn acl_revoke_admin(&mut self, role: String, account_id: AccountId) -> Option<bool>;

    /// Revokes admin permissions for `role` from the predecessor. Returns
    /// whether the predecessor was an admin for `role`.
    ///
    /// If an admin is revoked, the event described in
    /// [`Self::acl_revoke_admin`] will be emitted.
    fn acl_renounce_admin(&mut self, role: String) -> bool;

    /// Grants `role` to `account_id` provided that the predecessor has
    /// sufficient permissions, i.e. is an admin as defined by [`acl_is_admin`].
    ///
    /// In case of sufficient permissions, the returned `Some(bool)` indicates
    /// whether `account_id` is a new grantee of `role`. Without permissions,
    /// `None` is returned and internal state is not modified.
    ///
    /// If a role is granted, the following event will be emitted:
    ///
    /// ```json
    /// {
    ///    "standard":"AccessControllable",
    ///    "version":"1.0.0",
    ///    "event":"role_granted",
    ///    "data": {
    ///       "role":"<ROLE>",
    ///       "to":"<GRANTEE>",
    ///       "by":"<ADMIN>"
    ///    }
    /// }
    /// ```
    fn acl_grant_role(&mut self, role: String, account_id: AccountId) -> Option<bool>;

    /// Returns whether `account_id` has been granted `role`. Note that adding
    /// an account as (super-)admin for `role` does not make the account a
    /// grantee of `role`. Instead, `role` has to be granted explicitly.
    fn acl_has_role(&self, role: String, account_id: AccountId) -> bool;

    /// Revokes `role` from `account_id` provided that the predecessor has
    /// sufficient permissions, i.e. is an admin as defined by [`acl_is_admin`].
    ///
    /// In case of sufficient permissions, the returned `Some(bool)` indicates
    /// whether `account_id` was a grantee of `role`. Without permissions,
    /// `None` is returned and internal state is not modified.
    ///
    /// If a role is revoked, the following event will be emitted:
    ///
    /// ```json
    /// {
    ///    "standard":"AccessControllable",
    ///    "version":"1.0.0",
    ///    "event":"role_revoked",
    ///    "data": {
    ///       "role":"<ROLE>",
    ///       "from":"<GRANTEE>",
    ///       "by":"<ADMIN>"
    ///    }
    /// }
    /// ```
    fn acl_revoke_role(&mut self, role: String, account_id: AccountId) -> Option<bool>;

    /// Revokes `role` from the predecessor and returns whether it was a grantee
    /// of `role`.
    ///
    /// If a role is revoked, the event described in [`Self::acl_revoke_role`]
    /// will be emitted.
    fn acl_renounce_role(&mut self, role: String) -> bool;

    /// Returns whether `account_id` has been granted any of the `roles`.
    fn acl_has_any_role(&self, roles: Vec<String>, account_id: AccountId) -> bool;

    /// Enables paginated retrieval of super-admins. It returns up to `limit`
    /// super-admins and skips the first `skip` super-admins.
    fn acl_get_super_admins(&self, skip: u64, limit: u64) -> Vec<AccountId>;

    /// Enables paginated retrieval of admins of `role`. It returns up to
    /// `limit` admins and skips the first `skip` admins.
    fn acl_get_admins(&self, role: String, skip: u64, limit: u64) -> Vec<AccountId>;

    /// Enables paginated retrieval of grantees of `role`. It returns up to
    /// `limit` grantees and skips the first `skip` grantees.
    fn acl_get_grantees(&self, role: String, skip: u64, limit: u64) -> Vec<AccountId>;

    /// Convenience method that returns all [`PermissionedAccounts`].
    ///
    /// # Gas limits
    ///
    /// This function is eligible for view calls and while view calls are free for users, the
    /// underlying transaction is still subject to a [gas limit] defined by the RPC node.
    ///
    /// In use cases where gas cost matters, the data returned by this function can be retrieved
    /// more efficiently by a combination of the following:
    ///
    /// * Get roles with [`Self::acl_get_roles`].
    /// * Get (a subset) of permissioned accounts with [`Self::acl_get_super_admins`],
    /// [`Self::acl_get_admins`], or [`Self::acl_get_grantees`].
    ///
    /// [gas limit]: https://github.com/near/nearcore/pull/4381
    fn acl_get_permissioned_accounts(&self) -> PermissionedAccounts;
}

/// Collects super admin accounts and accounts that have been granted permissions defined by
/// `AccessControlRole`.
///
/// # Data structure
///
/// Assume `AccessControlRole` is derived for the following enum, which is then passed as `role`
/// attribute to `AccessControllable`.
///
/// ```rust
/// pub enum Role {
///     PauseManager,
///     UnpauseManager,
/// }
/// ```
///
/// Then the returned data has the following structure:
///
/// ```ignore
/// PermissionedAccounts {
///     super_admins: vec!["acc1.near", "acc2.near"],
///     roles: HashMap::from([
///         ("PauseManager", PermissionedAccountsPerRole {
///             admins: vec!["acc3.near", "acc4.near"],
///             grantees: vec!["acc5.near", "acc6.near"],
///         }),
///         ("UnpauseManager", PermissionedAccountsPerRole {
///             admins: vec!["acc7.near", "acc8.near"],
///             grantees: vec!["acc9.near", "acc10.near"],
///         }),
///     ])
/// }
/// ```
///
/// # Uniqueness and ordering
///
/// Account ids returned in vectors are unique but not ordered.
#[derive(Deserialize, Serialize, Debug)]
pub struct PermissionedAccounts {
    /// The accounts that have super admin permissions.
    pub super_admins: Vec<AccountId>,
    /// The admins and grantees of all roles.
    pub roles: HashMap<String, PermissionedAccountsPerRole>,
}

/// Collects all admins and grantees of a role.
///
/// # Uniqueness and ordering
///
/// Account ids returned in vectors are unique but not ordered.
#[derive(Deserialize, Serialize, Debug)]
pub struct PermissionedAccountsPerRole {
    /// The accounts that have admin permissions for the role.
    pub admins: Vec<AccountId>,
    /// The accounts that have been granted the role.
    pub grantees: Vec<AccountId>,
}

pub mod events {
    use crate::events::{AsEvent, EventMetadata};
    use near_sdk::serde::Serialize;
    use near_sdk::AccountId;

    const STANDARD: &str = "AccessControllable";
    const VERSION: &str = "1.0.0";

    /// Event emitted when an account is made super-admin.
    #[derive(Serialize, Clone)]
    #[serde(crate = "near_sdk::serde")]
    pub struct SuperAdminAdded {
        /// Account that was added as super-admin.
        pub account: AccountId,
        /// Account that added the super-admin.
        pub by: AccountId,
    }

    impl AsEvent<Self> for SuperAdminAdded {
        fn metadata(&self) -> EventMetadata<Self> {
            EventMetadata {
                standard: STANDARD.to_string(),
                version: VERSION.to_string(),
                event: "super_admin_added".to_string(),
                data: Some(self.clone()),
            }
        }
    }

    /// Event emitted when super-admin permissions are revoked.
    #[derive(Serialize, Clone)]
    #[serde(crate = "near_sdk::serde")]
    pub struct SuperAdminRevoked {
        /// Account from whom permissions were revoked.
        pub account: AccountId,
        /// Account that revoked the permissions.
        pub by: AccountId,
    }

    impl AsEvent<Self> for SuperAdminRevoked {
        fn metadata(&self) -> EventMetadata<Self> {
            EventMetadata {
                standard: STANDARD.to_string(),
                version: VERSION.to_string(),
                event: "super_admin_revoked".to_string(),
                data: Some(self.clone()),
            }
        }
    }

    /// Event emitted when an account is made admin.
    #[derive(Serialize, Clone)]
    #[serde(crate = "near_sdk::serde")]
    pub struct AdminAdded {
        /// The Role for which an admin was added.
        pub role: String,
        /// Account that was added as admin.
        pub account: AccountId,
        /// Account that added the admin.
        pub by: AccountId,
    }

    impl AsEvent<Self> for AdminAdded {
        fn metadata(&self) -> EventMetadata<Self> {
            EventMetadata {
                standard: STANDARD.to_string(),
                version: VERSION.to_string(),
                event: "admin_added".to_string(),
                data: Some(self.clone()),
            }
        }
    }

    /// Event emitted when admin permissions are revoked.
    #[derive(Serialize, Clone)]
    #[serde(crate = "near_sdk::serde")]
    pub struct AdminRevoked {
        /// The Role for which an admin was revoked.
        pub role: String,
        /// Account from whom permissions where revoked.
        pub account: AccountId,
        /// Account that revoked the admin.
        pub by: AccountId,
    }

    impl AsEvent<Self> for AdminRevoked {
        fn metadata(&self) -> EventMetadata<Self> {
            EventMetadata {
                standard: STANDARD.to_string(),
                version: VERSION.to_string(),
                event: "admin_revoked".to_string(),
                data: Some(self.clone()),
            }
        }
    }

    /// Event emitted when a role is granted to an account.
    #[derive(Serialize, Clone)]
    #[serde(crate = "near_sdk::serde")]
    pub struct RoleGranted {
        /// Role that was granted.
        pub role: String,
        /// Account that was granted the role.
        pub to: AccountId,
        /// Account that granted the role.
        pub by: AccountId,
    }

    impl AsEvent<Self> for RoleGranted {
        fn metadata(&self) -> EventMetadata<Self> {
            EventMetadata {
                standard: STANDARD.to_string(),
                version: VERSION.to_string(),
                event: "role_granted".to_string(),
                data: Some(self.clone()),
            }
        }
    }

    /// Event emitted when a role is revoked from an account.
    #[derive(Serialize, Clone)]
    #[serde(crate = "near_sdk::serde")]
    pub struct RoleRevoked {
        /// Role that was revoked.
        pub role: String,
        /// Account from whom the role was revoked.
        pub from: AccountId,
        /// Account that revoked the role.
        pub by: AccountId,
    }

    impl AsEvent<Self> for RoleRevoked {
        fn metadata(&self) -> EventMetadata<Self> {
            EventMetadata {
                standard: STANDARD.to_string(),
                version: VERSION.to_string(),
                event: "role_revoked".to_string(),
                data: Some(self.clone()),
            }
        }
    }
}
