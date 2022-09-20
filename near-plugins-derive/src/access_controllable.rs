use crate::access_control_role::new_bitflags_type_ident;
use darling::FromMeta;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::parse::Parser;
use syn::{parse_macro_input, AttributeArgs, ItemStruct};

#[derive(Debug, FromMeta)]
pub struct MacroArgs {
    #[darling(default)]
    storage_prefix: Option<String>,
    role_type: syn::Path,
}

const DEFAULT_STORAGE_PREFIX: &str = "__acl";
const DEFAULT_ACL_FIELD_NAME: &str = "__acl";
const DEFAULT_ACL_TYPE_NAME: &str = "__Acl";

const ERR_PARSE_BITFLAG: &str = "Value does not correspond to a permission";
const ERR_PARSE_ROLE: &str = "Value does not correspond to a role";

pub fn access_controllable(attrs: TokenStream, item: TokenStream) -> TokenStream {
    let attr_args = parse_macro_input!(attrs as AttributeArgs);
    let mut input: ItemStruct = parse_macro_input!(item);
    let acl_field = syn::Ident::new(DEFAULT_ACL_FIELD_NAME, Span::call_site());
    let acl_type = syn::Ident::new(DEFAULT_ACL_TYPE_NAME, Span::call_site());
    let bitflags_type = new_bitflags_type_ident(Span::call_site());
    if let Err(e) = inject_acl_field(&mut input, &acl_field, &acl_type) {
        return TokenStream::from(e.to_compile_error());
    }
    let ItemStruct { ident, .. } = input.clone();

    // TODO verify trait bounds on `role_enum`: BorshSerialize, BorshDeserialize, ...
    let macro_args = match MacroArgs::from_list(&attr_args) {
        Ok(args) => args,
        Err(e) => {
            return TokenStream::from(e.write_errors());
        }
    };
    let storage_prefix = macro_args
        .storage_prefix
        .unwrap_or_else(|| DEFAULT_STORAGE_PREFIX.to_string());
    let role_type = macro_args.role_type;

    let output = quote! {
        #input

        #[derive(::near_sdk::borsh::BorshDeserialize, ::near_sdk::borsh::BorshSerialize)]
        struct #acl_type {
            /// Stores permissions per account.
            permissions: ::near_sdk::collections::UnorderedMap<
                ::near_sdk::AccountId,
                #bitflags_type,
            >,
            /// Stores the set of accounts that bear a permission.
            bearers: ::near_sdk::collections::UnorderedMap<
                #bitflags_type,
                ::near_sdk::collections::UnorderedSet<::near_sdk::AccountId>,
            >,
        }

        impl Default for #acl_type {
            fn default() -> Self {
                let base_prefix = <#ident as AccessControllable>::acl_storage_prefix();
                Self {
                     permissions: ::near_sdk::collections::UnorderedMap::new(
                        __acl_storage_prefix(base_prefix, __AclStorageKey::Permissions),
                    ),
                    bearers: ::near_sdk::collections::UnorderedMap::new(
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
                .expect("Storage key should be serializable");
            [base, specifier.as_slice()].concat()
        }

        impl #acl_type {
            fn new_bearers_set(permission: #bitflags_type) -> ::near_sdk::collections::UnorderedSet<::near_sdk::AccountId> {
                let base_prefix = <#ident as AccessControllable>::acl_storage_prefix();
                let specifier = __AclStorageKey::BearersSet { permission };
                ::near_sdk::collections::UnorderedSet::new(__acl_storage_prefix(base_prefix, specifier))
            }

            fn get_or_init_permissions(&self, account_id: &::near_sdk::AccountId) -> #bitflags_type {
                match self.permissions.get(account_id) {
                    Some(permissions) => permissions,
                    None => <#bitflags_type>::empty(),
                }
            }

            fn grant_role_unchecked(&mut self, role: #role_type, account_id: &::near_sdk::AccountId) -> bool {
                let flag = <#bitflags_type>::from_bits(role.acl_permission())
                    .expect(#ERR_PARSE_BITFLAG);
                let mut permissions = self.get_or_init_permissions(account_id);

                let is_new_grantee = !permissions.contains(flag);
                if is_new_grantee {
                    permissions.insert(flag);
                    self.permissions.insert(account_id, &permissions);
                    self.add_bearer(flag, account_id);
                    // TODO emit event
                }

                is_new_grantee
            }

            fn has_role(&self, role: #role_type, account_id: &::near_sdk::AccountId) -> bool {
                match self.permissions.get(account_id) {
                    Some(permissions) => {
                        let flag = <#bitflags_type>::from_bits(role.acl_permission())
                            .expect(#ERR_PARSE_BITFLAG);
                        permissions.contains(flag)
                    }
                    None => false,
                }
            }

            /// Adds `account_id` to the set of `permission` bearers.
            fn add_bearer(&mut self, permission: #bitflags_type, account_id: &::near_sdk::AccountId) {
                let mut set = match self.bearers.get(&permission) {
                    Some(set) => set,
                    None => Self::new_bearers_set(permission),
                };
                if let true = set.insert(account_id) {
                    self.bearers.insert(&permission, &set);
                }
            }
        }

        // TODO control which functions are exposed externally
        // `near_bindgen` externally exposes functions in trait implementations
        // _even_ if they are not `pub`. This behavior is [documented] (but
        // still surprising IMO).
        //
        // [documented]: https://docs.near.org/sdk/rust/contract-interface/public-methods#exposing-trait-implementations
        #[near_bindgen]
        impl AccessControllable for #ident {
            fn acl_storage_prefix() -> &'static [u8] {
                (#storage_prefix).as_bytes()
            }

            fn acl_add_admin_unchecked(&mut self, role: String, account_id: ::near_sdk::AccountId) -> bool {
                false // TODO
            }

            fn acl_grant_role_unchecked(&mut self, role: String, account_id: ::near_sdk::AccountId) -> bool {
                let role = <#role_type>::try_from(role.as_str()).expect(#ERR_PARSE_ROLE);
                self.#acl_field.grant_role_unchecked(role, &account_id)
            }

            fn acl_has_role(&self, role: String, account_id: ::near_sdk::AccountId) -> bool {
                let role = <#role_type>::try_from(role.as_str()).expect(#ERR_PARSE_ROLE);
                self.#acl_field.has_role(role, &account_id)
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
