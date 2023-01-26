/// Represents permissions for the [`AccessControllable`](crate::AccessControllable) plugin.
pub trait AccessControlRole {
    /// Returns the names of all role variants.
    fn acl_role_variants() -> Vec<&'static str>;

    /// Returns the bitflag corresponding to the super admin permission.
    fn acl_super_admin_permission() -> u128;

    /// Returns the bitflag corresponding to the admin permission for the role.
    fn acl_admin_permission(self) -> u128;

    /// Returns the bitflag corresponding to the role's permission.
    fn acl_permission(self) -> u128;
}
