use near_sdk::AccountId;

/// # Representation of roles
///
/// This trait is unaware of the concrete type used to represent roles. It is
/// not possible to use a generic type `R` since `near-sdk` [does not support]
/// `impl` type parameters.
///
/// ```
/// // This is not possible:
/// impl<R> AccessControllable<R> for Contract {/* ... */}
/// ```
///
/// Instead, roles are represented by `u8`, which allows contract developers to
/// define their own enum whose variants are converted to `u8`.
///
/// [does not support]: https://github.com/near/near-sdk-rs/blob/9d99077c6acfde68c06845f2a1eb2b5ed7983401/near-sdk/compilation_tests/impl_generic.stderr
pub trait AccessControllable {
    fn acl_storage_prefix(&self) -> &[u8];

    /// Grants admin permissions for `role` to `account_id`, __without__
    /// checking permissions of the predecessor.
    ///
    /// Returns whether `account_id` was newly added to the admins for `role`.
    fn acl_add_admin_unchecked(&mut self, role: String, account_id: AccountId) -> bool;

    /// Grants `role` to `account_id` __without__ checking any permissions.
    /// Returns whether `role` was newly granted to `account_id`.
    fn acl_grant_role_unchecked(&mut self, role: String, account_id: AccountId) -> bool;

    /// Returns whether `account_id` has been granted `role`.
    fn acl_has_role(&self, role: String, account_id: AccountId) -> bool;
}
