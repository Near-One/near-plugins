use crate::access_control_role::new_bitflags_type_ident;
use crate::utils;
use crate::utils::{cratename, is_near_bindgen_wrapped_or_marshall};
use darling::FromMeta;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::parse::Parser;
use syn::{parse_macro_input, AttributeArgs, ItemFn, ItemStruct};

/// Defines attributes for the `access_controllable` macro.
#[derive(Debug, FromMeta)]
pub struct MacroArgs {
    #[darling(default)]
    storage_prefix: Option<String>,
    role_type: darling::util::PathList,
}

const DEFAULT_STORAGE_PREFIX: &str = "__acl";
const DEFAULT_ACL_FIELD_NAME: &str = "__acl";
const DEFAULT_ACL_TYPE_NAME: &str = "__Acl";

const ERR_PARSE_BITFLAG: &str = "Value does not correspond to a permission";
const ERR_PARSE_ROLE: &str = "Value does not correspond to a role";

/// Generates the token stream that implements `AccessControllable`.
pub fn access_controllable(attrs: TokenStream, item: TokenStream) -> TokenStream {
    let cratename = cratename();
    let attr_args = parse_macro_input!(attrs as AttributeArgs);
    let mut input: ItemStruct = parse_macro_input!(item);
    let acl_field = syn::Ident::new(DEFAULT_ACL_FIELD_NAME, Span::call_site());
    let acl_type = syn::Ident::new(DEFAULT_ACL_TYPE_NAME, Span::call_site());
    let bitflags_type = new_bitflags_type_ident(Span::call_site());
    if let Err(e) = inject_acl_field(&mut input, &acl_field, &acl_type) {
        return TokenStream::from(e.to_compile_error());
    }
    let ItemStruct { ident, .. } = input.clone();

    let macro_args = match MacroArgs::from_list(&attr_args) {
        Ok(args) => args,
        Err(e) => {
            return TokenStream::from(e.write_errors());
        }
    };
    let storage_prefix = macro_args
        .storage_prefix
        .unwrap_or_else(|| DEFAULT_STORAGE_PREFIX.to_string());
    assert!(
        macro_args.role_type.len() == 1,
        "role_type should be exactly one path"
    );
    let role_type = &macro_args.role_type[0];

    let output = quote! {
        #input

        #[derive(::near_sdk::borsh::BorshDeserialize, ::near_sdk::borsh::BorshSerialize)]
        struct #acl_type {
            /// Stores permissions per account.
            permissions: ::near_sdk::store::UnorderedMap<
                ::near_sdk::AccountId,
                #bitflags_type,
            >,
            /// Stores the set of accounts that bear a permission.
            bearers: ::near_sdk::store::UnorderedMap<
                #bitflags_type,
                ::near_sdk::store::UnorderedSet<::near_sdk::AccountId>,
            >,
        }

        impl Default for #acl_type {
            fn default() -> Self {
                let base_prefix = <#ident as AccessControllable>::acl_storage_prefix();
                Self {
                     permissions: ::near_sdk::store::UnorderedMap::new(
                        __acl_storage_prefix(base_prefix, __AclStorageKey::Permissions),
                    ),
                    bearers: ::near_sdk::store::UnorderedMap::new(
                        __acl_storage_prefix(base_prefix, __AclStorageKey::Bearers),
                    ),
                }
            }
        }

        /// Used to make storage prefixes unique. Not to be used directly,
        /// instead it should be prepended to the storage prefix specified by
        /// the user.
        #[derive(::near_sdk::borsh::BorshSerialize)]
        enum __AclStorageKey {
            Permissions,
            Bearers,
            BearersSet { permission: #bitflags_type },
        }

        /// Generates a prefix by concatenating the input parameters.
        fn __acl_storage_prefix(base: &[u8], specifier: __AclStorageKey) -> Vec<u8> {
            let specifier = specifier
                .try_to_vec()
                .unwrap_or_else(|_| ::near_sdk::env::panic_str("Storage key should be serializable"));
            [base, specifier.as_slice()].concat()
        }

        impl #acl_type {
            fn new_bearers_set(permission: #bitflags_type) -> ::near_sdk::store::UnorderedSet<::near_sdk::AccountId> {
                let base_prefix = <#ident as AccessControllable>::acl_storage_prefix();
                let specifier = __AclStorageKey::BearersSet { permission };
                ::near_sdk::store::UnorderedSet::new(__acl_storage_prefix(base_prefix, specifier))
            }

            fn get_or_insert_permissions(&mut self, account_id: ::near_sdk::AccountId) -> &mut #bitflags_type {
                self.permissions.entry(account_id).or_insert_with(|| #bitflags_type::empty())
            }

            fn init_super_admin(&mut self, account_id: &::near_sdk::AccountId) -> bool {
                let flag = <#bitflags_type>::from_bits(<#role_type>::acl_super_admin_permission())
                    .unwrap_or_else(|| ::near_sdk::env::panic_str(#ERR_PARSE_BITFLAG));
                // TODO use self.get_bearers() once the following PR is merged
                // https://github.com/aurora-is-near/near-plugins/pull/12
                let number_super_admins = match self.bearers.get(&flag) {
                    None => 0,
                    Some(bearers) => bearers.len(),
                };
                if number_super_admins > 0 {
                    return false;
                }
                let res = self.add_super_admin_unchecked(account_id);
                ::near_sdk::require!(res, "Failed to init super-admin.");
                res
            }

            /// Makes `account_id` a super-admin __without__ checking any permissions.
            /// It returns whether `account_id` is a new super-admin.
            ///
            /// Note that there may be zero or more super-admins.
            fn add_super_admin_unchecked(&mut self, account_id: &::near_sdk::AccountId) -> bool {
                let flag = <#bitflags_type>::from_bits(<#role_type>::acl_super_admin_permission())
                    .unwrap_or_else(|| ::near_sdk::env::panic_str(#ERR_PARSE_BITFLAG));
                let mut permissions = self.get_or_insert_permissions(account_id.clone());

                let is_new_super_admin = !permissions.contains(flag);
                if is_new_super_admin {
                    permissions.insert(flag);
                    self.add_bearer(flag, account_id);

                    let event = #cratename::access_controllable::events::SuperAdminAdded {
                        account: account_id.clone(),
                        by: ::near_sdk::env::predecessor_account_id(),
                    };
                    #cratename::events::AsEvent::emit(&event);
                }

                is_new_super_admin
            }

            fn is_super_admin(&self, account_id: &::near_sdk::AccountId) -> bool {
                let permissions = {
                    match self.permissions.get(account_id) {
                        Some(permissions) => permissions,
                        None => return false,
                    }
                };
                let super_admin = <#bitflags_type>::from_bits(<#role_type>::acl_super_admin_permission())
                    .unwrap_or_else(|| ::near_sdk::env::panic_str(#ERR_PARSE_BITFLAG));
                permissions.contains(super_admin)
            }

            /// Revokes super-admin permissions from `account_id` without checking any
            /// permissions. It returns whether `account_id` was a super-admin.
            fn revoke_super_admin_unchecked(&mut self, account_id: &::near_sdk::AccountId) -> bool {
                let flag = <#bitflags_type>::from_bits(<#role_type>::acl_super_admin_permission())
                    .unwrap_or_else(|| ::near_sdk::env::panic_str(#ERR_PARSE_BITFLAG));
                let mut permissions = match self.permissions.get_mut(account_id) {
                    Some(permissions) => permissions,
                    None => return false, // nothing to do, account has no permissions
                };

                let was_super_admin = permissions.contains(flag);
                if was_super_admin {
                    permissions.remove(flag);
                    self.remove_bearer(flag, account_id);

                    let event = #cratename::access_controllable::events::SuperAdminRevoked {
                        account: account_id.clone(),
                        by: ::near_sdk::env::predecessor_account_id(),
                    };
                    #cratename::events::AsEvent::emit(&event);
                }

                was_super_admin
            }

            fn add_admin(&mut self, role: #role_type, account_id: &::near_sdk::AccountId) -> Option<bool> {
                if !self.is_admin(role, &::near_sdk::env::predecessor_account_id()) {
                    return None;
                }
                Some(self.add_admin_unchecked(role, account_id))
            }

            /// Makes `account_id` an admin for role, __without__ checking any
            /// permissions. Returns whether `account_id` is a new admin for `role`.
            ///
            /// Note that any role may have multiple (or zero) admins.
            fn add_admin_unchecked(&mut self, role: #role_type, account_id: &::near_sdk::AccountId) -> bool {
                let flag = <#bitflags_type>::from_bits(role.acl_admin_permission())
                    .unwrap_or_else(|| ::near_sdk::env::panic_str(#ERR_PARSE_BITFLAG));
                let mut permissions = self.get_or_insert_permissions(account_id.clone());

                let is_new_admin = !permissions.contains(flag);
                if is_new_admin {
                    permissions.insert(flag);
                    self.add_bearer(flag, account_id);

                    let event = #cratename::access_controllable::events::AdminAdded {
                        role: role.into(),
                        account: account_id.clone(),
                        by: ::near_sdk::env::predecessor_account_id(),
                    };
                    #cratename::events::AsEvent::emit(&event);
                }

                is_new_admin
            }

            fn is_admin(&self, role: #role_type, account_id: &::near_sdk::AccountId) -> bool {
                let permissions = {
                    match self.permissions.get(account_id) {
                        Some(permissions) => permissions,
                        None => return false,
                    }
                };
                let super_admin = <#bitflags_type>::from_bits(<#role_type>::acl_super_admin_permission())
                    .unwrap_or_else(|| ::near_sdk::env::panic_str(#ERR_PARSE_BITFLAG));
                let role_admin = <#bitflags_type>::from_bits(role.acl_admin_permission())
                    .unwrap_or_else(|| ::near_sdk::env::panic_str(#ERR_PARSE_BITFLAG));
                permissions.contains(super_admin) || permissions.contains(role_admin)
            }

            fn revoke_admin(&mut self, role: #role_type, account_id: &::near_sdk::AccountId) -> Option<bool> {
                if !self.is_admin(role, &::near_sdk::env::predecessor_account_id()) {
                    return None;
                }
                Some(self.revoke_admin_unchecked(role, account_id))
            }

            fn renounce_admin(&mut self, role: #role_type) -> bool {
                self.revoke_admin_unchecked(role, &::near_sdk::env::predecessor_account_id())
            }

            /// Revokes admin permissions from `account_id` __without__ checking any
            /// permissions. Returns whether `account_id` was an admin for `role`.
            fn revoke_admin_unchecked(&mut self, role: #role_type, account_id: &::near_sdk::AccountId) -> bool {
                let flag = <#bitflags_type>::from_bits(role.acl_admin_permission())
                    .unwrap_or_else(|| ::near_sdk::env::panic_str(#ERR_PARSE_BITFLAG));
                let mut permissions = match self.permissions.get_mut(account_id) {
                    Some(permissions) => permissions,
                    None => return false, // nothing to do, account has no permissions
                };

                let was_admin = permissions.contains(flag);
                if was_admin {
                    permissions.remove(flag);
                    self.remove_bearer(flag, account_id);

                    let event = #cratename::access_controllable::events::AdminRevoked {
                        role: role.into(),
                        account: account_id.clone(),
                        by: ::near_sdk::env::predecessor_account_id(),
                    };
                    #cratename::events::AsEvent::emit(&event);
                }

                was_admin
            }

            fn grant_role(&mut self, role: #role_type, account_id: &::near_sdk::AccountId) -> Option<bool> {
                if !self.is_admin(role, &::near_sdk::env::predecessor_account_id()) {
                    return None;
                }
                Some(self.grant_role_unchecked(role, account_id))
            }

            /// Grants `role` to `account_id` __without__ checking any permissions.
            /// Returns whether `role` was newly granted to `account_id`.
            fn grant_role_unchecked(&mut self, role: #role_type, account_id: &::near_sdk::AccountId) -> bool {
                let flag = <#bitflags_type>::from_bits(role.acl_permission())
                    .unwrap_or_else(|| ::near_sdk::env::panic_str(#ERR_PARSE_BITFLAG));
                let mut permissions = self.get_or_insert_permissions(account_id.clone());

                let is_new_grantee = !permissions.contains(flag);
                if is_new_grantee {
                    permissions.insert(flag);
                    self.add_bearer(flag, account_id);

                    let event = #cratename::access_controllable::events::RoleGranted {
                        role: role.into(),
                        by: ::near_sdk::env::predecessor_account_id(),
                        to: account_id.clone(),
                    };
                    #cratename::events::AsEvent::emit(&event);
                }

                is_new_grantee
            }

            fn revoke_role(&mut self, role: #role_type, account_id: &::near_sdk::AccountId) -> Option<bool> {
                if !self.is_admin(role, &::near_sdk::env::predecessor_account_id()) {
                    return None;
                }
                Some(self.revoke_role_unchecked(role, account_id))
            }

            fn renounce_role(&mut self, role: #role_type) -> bool {
                self.revoke_role_unchecked(role, &::near_sdk::env::predecessor_account_id())
            }

            fn revoke_role_unchecked(&mut self, role: #role_type, account_id: &::near_sdk::AccountId) -> bool {
                let flag = <#bitflags_type>::from_bits(role.acl_permission())
                    .unwrap_or_else(|| ::near_sdk::env::panic_str(#ERR_PARSE_BITFLAG));
                let mut permissions = match self.permissions.get_mut(account_id) {
                    Some(permissions) => permissions,
                    None => return false, // nothing to do, account has no permissions
                };

                let was_grantee = permissions.contains(flag);
                if was_grantee {
                    permissions.remove(flag);
                    self.remove_bearer(flag, account_id);

                    let event = #cratename::access_controllable::events::RoleRevoked {
                        role: role.into(),
                        from: account_id.clone(),
                        by: ::near_sdk::env::predecessor_account_id(),
                    };
                    #cratename::events::AsEvent::emit(&event);
                }

                was_grantee
            }

            fn has_role(&self, role: #role_type, account_id: &::near_sdk::AccountId) -> bool {
                match self.permissions.get(account_id) {
                    Some(permissions) => {
                        let flag = <#bitflags_type>::from_bits(role.acl_permission())
                            .unwrap_or_else(|| ::near_sdk::env::panic_str(#ERR_PARSE_BITFLAG));
                        permissions.contains(flag)
                    }
                    None => false,
                }
            }

            fn has_any_role(
                &self, roles: Vec<#role_type>,
                account_id: &::near_sdk::AccountId
            ) -> bool {
                // Create a bitflags value with active bits for all `roles`.
                let target = roles
                    .iter()
                    .map(|role| {
                        <#bitflags_type>::from_bits(role.acl_permission())
                            .unwrap_or_else(|| ::near_sdk::env::panic_str(#ERR_PARSE_BITFLAG))
                    })
                    .fold(
                        <#bitflags_type>::empty(),
                        |acc, x| acc | x,
                    );
                self.has_any_permission(target, account_id)
            }

            fn has_any_permission(&self, target: #bitflags_type, account_id: &::near_sdk::AccountId) -> bool {
                let permissions = match self.permissions.get(account_id) {
                    Some(&permissions) => permissions,
                    None => return false,
                };
                target.intersects(permissions)
            }

            /// Adds `account_id` to the set of `permission` bearers.
            ///
            /// # Panics
            ///
            /// Panics if `permission` has more than one active bit. The type of
            /// permission defines only flags which have one active bit. Still,
            /// developers might call this function with a `permission` that has
            /// multiple active bits. In that case, the panic prevents polluting
            /// state.
            fn add_bearer(&mut self, permission: #bitflags_type, account_id: &::near_sdk::AccountId) {
                ::near_sdk::require!(
                    permission.bits().is_power_of_two(),
                    "Adding a bearer is allowed only for permissions with exactly one active bit"
                );
                let mut set = self.bearers.entry(permission).or_insert_with(|| {
                    Self::new_bearers_set(permission)
                });
                set.insert(account_id.clone());
            }

            /// Enables paginated retrieval of bearers. Returns up to `limit`
            /// bearers of `permission`, skipping the first `skip` items.
            ///
            /// # Panics
            ///
            /// Panics if `skip` or `limit` are outside the range of `usize`.
            fn get_bearers(&self, permission: #bitflags_type, skip: u64, limit: u64) -> Vec<::near_sdk::AccountId> {
                let skip: usize = ::std::convert::TryFrom::try_from(skip).unwrap_or_else(|_| ::near_sdk::env::panic_str("skip should be in the range of usize"));
                let limit: usize = ::std::convert::TryFrom::try_from(limit).unwrap_or_else(|_| ::near_sdk::env::panic_str("limit should be in the range of usize"));
                let set = match self.bearers.get(&permission) {
                    Some(set) => set,
                    None => return vec![],
                };
                set.iter().skip(skip).take(limit).cloned().collect()
            }

            /// Removes `account_id` from the set of `permission` bearers.
            fn remove_bearer(&mut self, permission: #bitflags_type, account_id: &::near_sdk::AccountId) {
                // If `permission` is invalid (more than one active bit), this
                // function is a no-op, due to the check in `add_bearer`.
                let mut set = match self.bearers.get_mut(&permission) {
                    Some(set) => set,
                    None => return,
                };
                set.remove(account_id);
            }
        }

        // Note that `#[near-bindgen]` exposes non-public functions in trait
        // implementations. This is [documented] behavior. Therefore some
        // functions are made `#[private]` despite _not_ being public.
        //
        // [documented]: https://docs.near.org/sdk/rust/contract-interface/public-methods#exposing-trait-implementations
        #[near_bindgen]
        impl AccessControllable for #ident {
            fn acl_storage_prefix() -> &'static [u8] {
                (#storage_prefix).as_bytes()
            }

            #[private]
            fn acl_init_super_admin(&mut self, account_id: ::near_sdk::AccountId) -> bool {
                self.#acl_field.init_super_admin(&account_id)
            }

            fn acl_is_super_admin(&self, account_id: ::near_sdk::AccountId) -> bool {
                self.#acl_field.is_super_admin(&account_id)
            }

            fn acl_add_admin(&mut self, role: String, account_id: ::near_sdk::AccountId) -> Option<bool> {
                let role: #role_type = ::std::convert::TryFrom::try_from(role.as_str()).unwrap_or_else(|_| ::near_sdk::env::panic_str(#ERR_PARSE_ROLE));
                self.#acl_field.add_admin(role, &account_id)
            }

            fn acl_is_admin(&self, role: String, account_id: ::near_sdk::AccountId) -> bool {
                let role: #role_type = ::std::convert::TryFrom::try_from(role.as_str()).unwrap_or_else(|_| ::near_sdk::env::panic_str(#ERR_PARSE_ROLE));
                self.#acl_field.is_admin(role, &account_id)
            }

            fn acl_revoke_admin(&mut self, role: String, account_id: ::near_sdk::AccountId) -> Option<bool> {
                let role: #role_type = ::std::convert::TryFrom::try_from(role.as_str()).unwrap_or_else(|_| ::near_sdk::env::panic_str(#ERR_PARSE_ROLE));
                self.#acl_field.revoke_admin(role, &account_id)
            }

            fn acl_renounce_admin(&mut self, role: String) -> bool {
                let role: #role_type = ::std::convert::TryFrom::try_from(role.as_str()).unwrap_or_else(|_| ::near_sdk::env::panic_str(#ERR_PARSE_ROLE));
                self.#acl_field.renounce_admin(role)
            }

            fn acl_revoke_role(&mut self, role: String, account_id: ::near_sdk::AccountId) -> Option<bool> {
                let role: #role_type = ::std::convert::TryFrom::try_from(role.as_str()).unwrap_or_else(|_| ::near_sdk::env::panic_str(#ERR_PARSE_ROLE));
                self.#acl_field.revoke_role(role, &account_id)
            }

            fn acl_renounce_role(&mut self, role: String) -> bool {
                let role: #role_type = ::std::convert::TryFrom::try_from(role.as_str()).unwrap_or_else(|_| ::near_sdk::env::panic_str(#ERR_PARSE_ROLE));
                self.#acl_field.renounce_role(role)
            }

            fn acl_grant_role(&mut self, role: String, account_id: ::near_sdk::AccountId) -> Option<bool> {
                let role: #role_type = ::std::convert::TryFrom::try_from(role.as_str()).unwrap_or_else(|_| ::near_sdk::env::panic_str(#ERR_PARSE_ROLE));
                self.#acl_field.grant_role(role, &account_id)
            }


            fn acl_has_role(&self, role: String, account_id: ::near_sdk::AccountId) -> bool {
                let role: #role_type = ::std::convert::TryFrom::try_from(role.as_str()).unwrap_or_else(|_| ::near_sdk::env::panic_str(#ERR_PARSE_ROLE));
                self.#acl_field.has_role(role, &account_id)
            }

            fn acl_has_any_role(&self, roles: Vec<String>, account_id: ::near_sdk::AccountId) -> bool {
                let roles: Vec<#role_type> = roles
                    .iter()
                    .map(|role| {
                        ::std::convert::TryFrom::try_from(role.as_str()).unwrap_or_else(|_| ::near_sdk::env::panic_str(#ERR_PARSE_ROLE))
                    })
                    .collect();
                self.#acl_field.has_any_role(roles, &account_id)
            }

            fn acl_get_super_admins(&self, skip: u64, limit: u64) -> Vec<::near_sdk::AccountId> {
                let permission = <#bitflags_type>::from_bits(
                    <#role_type>::acl_super_admin_permission()
                )
                .unwrap_or_else(|| ::near_sdk::env::panic_str(#ERR_PARSE_BITFLAG));
                self.#acl_field.get_bearers(permission, skip, limit)
            }

            fn acl_get_admins(&self, role: String, skip: u64, limit: u64) -> Vec<::near_sdk::AccountId> {
                let role: #role_type = ::std::convert::TryFrom::try_from(role.as_str()).unwrap_or_else(|_| ::near_sdk::env::panic_str(#ERR_PARSE_ROLE));
                let permission = <#bitflags_type>::from_bits(role.acl_admin_permission())
                    .unwrap_or_else(|| ::near_sdk::env::panic_str(#ERR_PARSE_BITFLAG));
                self.#acl_field.get_bearers(permission, skip, limit)
            }

            fn acl_get_grantees(&self, role: String, skip: u64, limit: u64) -> Vec<::near_sdk::AccountId> {
                let role: #role_type = ::std::convert::TryFrom::try_from(role.as_str()).unwrap_or_else(|_| ::near_sdk::env::panic_str(#ERR_PARSE_ROLE));
                let permission = <#bitflags_type>::from_bits(role.acl_permission())
                    .unwrap_or_else(|| ::near_sdk::env::panic_str(#ERR_PARSE_BITFLAG));
                self.#acl_field.get_bearers(permission, skip, limit)
            }
        }
    };

    output.into()
}

fn inject_acl_field(
    item: &mut ItemStruct,
    field_name: &syn::Ident,
    acl_type: &syn::Ident,
) -> Result<(), syn::Error> {
    let fields = match item.fields {
        syn::Fields::Named(ref mut fields) => fields,
        _ => {
            return Err(syn::Error::new(
                item.ident.span(),
                "Expected to have named fields",
            ))
        }
    };

    fields.named.push(syn::Field::parse_named.parse2(quote! {
        #field_name: #acl_type
    })?);
    Ok(())
}

/// Defines attributes for the `access_control_any` macro.
#[derive(Debug, FromMeta)]
pub struct MacroArgsAny {
    roles: darling::util::PathList,
}

/// Generates the token stream for the `access_control_any` macro.
pub fn access_control_any(attrs: TokenStream, item: TokenStream) -> TokenStream {
    let attr_args = parse_macro_input!(attrs as AttributeArgs);
    let cloned_item = item.clone();
    let input: ItemFn = parse_macro_input!(cloned_item);
    if is_near_bindgen_wrapped_or_marshall(&input) {
        return item;
    }

    let function_name = input.sig.ident.to_string();

    let macro_args = match MacroArgsAny::from_list(&attr_args) {
        Ok(args) => args,
        Err(e) => {
            return TokenStream::from(e.write_errors());
        }
    };
    let roles = macro_args.roles;
    assert!(roles.len() > 0, "Specify at least one role");

    // TODO optimize case `roles.len() == 1` (speed up expected common case)
    let acl_check = quote! {
        let __acl_any_roles: Vec<&str> = vec![#(#roles.into()),*];
        let __acl_any_roles_ser: Vec<String> =
            __acl_any_roles.iter().map(|&role| role.into()).collect();
        let __acl_any_account_id = ::near_sdk::env::predecessor_account_id();
        if !self.acl_has_any_role(__acl_any_roles_ser, __acl_any_account_id) {
            let message = format!(
                "Insufficient permissions for method {} restricted by access control. Requires one of these roles: {:?}",
                #function_name,
                __acl_any_roles,
            );
            near_sdk::env::panic_str(&message);
        }
    };

    utils::add_extra_code_to_fn(&input, acl_check)
}
