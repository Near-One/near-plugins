use near_sdk::AccountId;

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
    /// Returns the storage prefix for collections related to access control. By
    /// default `b"__acl"` is used.
    ///
    /// Attribute `storage_prefix` can be used to set a different prefix:
    ///
    /// ```ignore
    /// #[access_controllable(storage_prefix="CUSTOM_KEY")]
    /// struct Contract { /* ... */}
    /// ```
    fn acl_storage_prefix() -> &'static [u8];

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
    ///
    /// Despite the restrictions of this method, there might be multiple
    /// super-admins. Adding more than one admin requires the use of internal
    /// methods. The default implementation of `AccessControllable` provided by
    /// this trait offers `add_super_admin_unchecked.`
    fn acl_init_super_admin(&mut self, account_id: AccountId) -> bool;

    /// Returns whether `account_id` is a super-admin. A super-admin has admin
    /// permissions for every role. However, a super-admin is not considered
    /// grantee of any role.
    fn acl_is_super_admin(&self, account_id: AccountId) -> bool;

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
    /// Note that adding an account as admin for `role` does not make that
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
    /// an account as (super-)admin for `role` does not make that account a
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
}

pub mod events {
    use crate::events::{AsEvent, EventMetadata};
    use near_sdk::serde::Serialize;
    use near_sdk::AccountId;

    const STANDARD: &str = "AccessControllable";
    const VERSION: &str = "1.0.0";

    /// Event emitted when an accout is made super-admin.
    #[derive(Serialize, Clone)]
    #[serde(crate = "near_sdk::serde")]
    pub struct SuperAdminAdded {
        /// Account that was added as super-admin.
        pub account: AccountId,
        /// Account that added the super-admin.
        pub by: AccountId,
    }

    impl AsEvent<SuperAdminAdded> for SuperAdminAdded {
        fn metadata(&self) -> EventMetadata<SuperAdminAdded> {
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

    impl AsEvent<SuperAdminRevoked> for SuperAdminRevoked {
        fn metadata(&self) -> EventMetadata<SuperAdminRevoked> {
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

    impl AsEvent<AdminAdded> for AdminAdded {
        fn metadata(&self) -> EventMetadata<AdminAdded> {
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

    impl AsEvent<AdminRevoked> for AdminRevoked {
        fn metadata(&self) -> EventMetadata<AdminRevoked> {
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

    impl AsEvent<RoleGranted> for RoleGranted {
        fn metadata(&self) -> EventMetadata<RoleGranted> {
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

    impl AsEvent<RoleRevoked> for RoleRevoked {
        fn metadata(&self) -> EventMetadata<RoleRevoked> {
            EventMetadata {
                standard: STANDARD.to_string(),
                version: VERSION.to_string(),
                event: "role_revoked".to_string(),
                data: Some(self.clone()),
            }
        }
    }
}
