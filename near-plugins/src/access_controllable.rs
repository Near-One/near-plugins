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
    /// Returns the storage prefix for collections related to access control.
    fn acl_storage_prefix() -> &'static [u8];

    /// Makes `account_id` an admin provided that the predecessor has sufficient
    /// permissions, i.e. is an admin as defined by [`acl_is_admin`].
    ///
    /// In case of sufficient permissions, the returned `Some(bool)` indicates
    /// whether `account_id` is a new admin for `role`. Without permissions,
    /// `None` is returned and internal state is not modified.
    ///
    /// Note that any role may have multiple (or zero) admins.
    fn acl_add_admin(&mut self, role: String, account_id: AccountId) -> Option<bool>;

    /// Makes `account_id` an admin for role, __without__ checking any
    /// permissions. Returns whether `account_id` is a new admin for `role`.
    ///
    /// Note that any role may have multiple (or zero) admins.
    ///
    /// This method is `#[private]` in the implementation provided by this
    /// crate.
    fn acl_add_admin_unchecked(&mut self, role: String, account_id: AccountId) -> bool;

    /// Returns whether `account_id` is an admin for `role`. Super-admins are
    /// admins for _every_ role.
    fn acl_is_admin(&self, role: String, account_id: AccountId) -> bool;

    /// Revoke admin permissions for `role` from `account_id` provided that the
    /// predecessor has sufficient permissions, i.e. is an admin as defined by
    /// [`acl_is_admin`].
    ///
    /// In case of sufficient permissions, the returned `Some(bool)` indicates
    /// whether `account_id` was an admin for `role`. Without permissions,
    /// `None` is returned and internal state is not modified.
    fn acl_revoke_admin(&mut self, role: String, account_id: AccountId) -> Option<bool>;

    /// Revokes admin permissions for `role` from the predecessor. Returns
    /// whether the predecessor was an admin for `role`.
    fn acl_renounce_admin(&mut self, role: String) -> bool;

    /// Revokes admin permissions from `account_id` __without__ checking any
    /// permissions. Returns whether `account_id` was an admin for `role`.
    ///
    /// This method is `#[private]` in the implementation provided by this
    /// crate.
    fn acl_revoke_admin_unchecked(&mut self, role: String, account_id: AccountId) -> bool;

    /// Grants `role` to `account_id` __without__ checking any permissions.
    /// Returns whether `role` was newly granted to `account_id`.
    ///
    /// This method is `#[private]` in the implementation provided by this
    /// crate.
    fn acl_grant_role_unchecked(&mut self, role: String, account_id: AccountId) -> bool;

    /// Returns whether `account_id` has been granted `role`.
    fn acl_has_role(&self, role: String, account_id: AccountId) -> bool;

    /// Returns whether `account_id` has been granted any of the `roles`.
    fn acl_has_any_role(&self, roles: Vec<String>, account_id: AccountId) -> bool;
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
}
