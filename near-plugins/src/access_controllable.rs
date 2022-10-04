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

    /// Grants `role` to `account_id` __without__ checking any permissions.
    /// Returns whether `role` was newly granted to `account_id`.
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
