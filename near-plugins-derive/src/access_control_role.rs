//! Implements `AccessControlRole` for an enum.
//!
//! The conversion of enum variants to bitflags representable by `u128` is the
//! key part of this implementation. Assume the trait is derived on the
//! following enum:
//!
//! ```ignore
//! #[derive(AccessControlRole)]
//! pub enum Role {
//!     LevelA,
//!     LevelB,
//!     LevelC,
//! }
//! ```
//!
//! This results in the following bitflags:
//! ```ignore
//! bitflags! {
//!     struct RoleFlags: u128 {
//!         const __SUPER_ADMIN = 1u128 << 0;
//!         const LEVELA        = 1u128 << 1;
//!         const LEVELA_ADMIN  = 1u128 << 2;
//!         const LEVELB        = 1u128 << 3;
//!         const LEVELB_ADMIN  = 1u128 << 4;
//!         const LEVELC        = 1u128 << 5;
//!         const LEVELC_ADMIN  = 1u128 << 6;
//!     }
//! }
//! ```
//!
//! The mapping between enum variants and bitflag has these properties:
//!
//! - Each flag has exactly one bit with value 1.
//! - A bitflag `1u128 << x` with odd `x` represents a role permission.
//! - A bitflag `1u128 << x` with even `x` represents an admin permission.
//! - Shifting a role's 1-bit to the left by one position yields the
//!   corresponding admin permission.
//!
//! The last property aims to facilitate migrations which add or remove enum
//! variants.

use crate::utils::cratename;
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use std::convert::TryFrom;
use syn::{parse_macro_input, ItemEnum};

/// Roles as are represented by enum variants which are, in turn, represented by
/// `u128` bitflags. Each variant requires two flags, one for the role itself
/// and one for the corresponding admin permission. This would allow for 64
/// roles. However, one flag is reserved for `__SUPER_ADMIN`, leaving 127
/// bits that can fit 63 roles.
pub const MAX_ROLE_VARIANTS: u8 = 63;

const DEFAULT_SUPER_ADMIN_NAME: &str = "__SUPER_ADMIN";
const DEFAULT_BITFLAGS_TYPE_NAME: &str = "RoleFlags";
const DEFAULT_BOUNDCHECKER_TYPE_NAME: &str = "__AclBoundchecker";

/// Generates the token stream that implements `AccessControlRole`.
pub fn derive_access_control_role(input: TokenStream) -> TokenStream {
    // This derive doesn't take attributes, so no need to use `darling`.
    let cratename = cratename();
    let input: ItemEnum = parse_macro_input!(input);
    let ItemEnum {
        ident, variants, ..
    } = input;

    let variant_idents = variants.into_iter().map(|v| v.ident).collect::<Vec<_>>();
    assert!(
        variant_idents.len() <= usize::from(MAX_ROLE_VARIANTS),
        "The number of enum variants should not exceed MAX_ROLE_VARIANTS",
    );
    let variant_idxs: Vec<_> =
        (0..u8::try_from(variant_idents.len()).expect("Too many enum variants")).collect();
    let variant_names: Vec<_> = variant_idents.iter().map(|v| format!("{v}")).collect();

    let boundchecker_type = Ident::new(DEFAULT_BOUNDCHECKER_TYPE_NAME, ident.span());
    let bitflags_type_ident = new_bitflags_type_ident(Span::call_site());
    let bitflags_idents = bitflags_idents(variant_names.as_ref(), bitflags_type_ident.span());
    let bitflags_idxs: Vec<_> =
        (0..u8::try_from(bitflags_idents.len()).expect("Too many bitflags")).collect();

    let output = quote! {
        // Ensure #ident satisfies bounds required for acl. This is done
        // explicitly to provide a clear error message to developers whose
        // enum doesn't satisfy the required bounds.
        //
        // Without this explicit check, compilation would still fail if a bound
        // is not satisfied. Though with less a clear error message.
        struct #boundchecker_type<T: Copy + Clone> {
            _marker: std::marker::PhantomData<T>,
        }
        impl<T: Copy + Clone> #boundchecker_type<T> {
            fn new() -> Self {
                Self {  _marker: Default::default() }
            }
        }
        impl #ident {
            fn check_bounds() {
                // Compilation will fail if #ident doesn't satisfy above bounds.
                let _x = #boundchecker_type::<#ident>::new();
            }
        }

        impl From<#ident> for u8 {
            fn from(value: #ident) -> Self {
                match value {
                    #(
                        #ident::#variant_idents => #variant_idxs,
                    )*
                }
            }
        }

        impl std::convert::TryFrom<u8> for #ident {
            type Error = &'static str;

            fn try_from(value: u8) -> Result<Self, Self::Error> {
                match value {
                    #(
                        #variant_idxs => Ok(#ident::#variant_idents),
                    )*
                    _ => Err("Value does not correspond to a variant"),
                }
            }
        }

        impl From<#ident> for &'static str {
            fn from(value: #ident) -> Self {
                match value {
                    #(
                        #ident::#variant_idents => #variant_names,
                    )*
                }
            }
        }

        impl From<#ident> for String {
            fn from(value: #ident) -> Self {
                match value {
                    #(
                        #ident::#variant_idents => #variant_names.to_string(),
                    )*
                }
            }
        }

        impl std::convert::TryFrom<&str> for #ident {
            type Error = &'static str;

            fn try_from(value: &str) -> Result<#ident, Self::Error> {
                match value {
                    #(
                        #variant_names => Ok(#ident::#variant_idents),
                    )*
                    _ => Err("Value does not correspond to a variant"),
                }
            }
        }

        /// Panics if `n` is too large.
        fn safe_leftshift(value: u128, n: u8) -> u128 {
            value
                .checked_shl(n.into())
                .unwrap_or_else(|| near_sdk::env::panic_str("Too many enum variants to be represented by bitflags"))
        }

        impl #cratename::AccessControlRole for #ident {
            fn acl_role_variants() -> Vec<&'static str> {
                vec![
                    #(#variant_names,)*
                ]
            }

            fn acl_super_admin_permission() -> u128 {
                // See module documentation.
                1 // corresponds to safe_leftshift(1, 0)
            }

            fn acl_permission(self) -> u128 {
                // Shift 1u128 left by an odd number, see module documentation.
                let n = (u8::from(self) + 1)
                    .checked_mul(2).unwrap_or_else(|| near_sdk::env::panic_str("Too many enum variants")) - 1;
                safe_leftshift(1, n)
            }

            fn acl_admin_permission(self) -> u128 {
                // Shift 1u128 left by an even number, see module documentation.
                let n = (u8::from(self) + 1)
                    .checked_mul(2)
                    .unwrap_or_else(|| near_sdk::env::panic_str("Too many enum variants"));
                safe_leftshift(1, n)
            }
        }

        #cratename::bitflags::bitflags! {
            /// Encodes permissions for roles and admins.
            #[derive(
                Default,
                near_sdk::borsh::BorshDeserialize,
                near_sdk::borsh::BorshSerialize,
            )]
            #[borsh(crate = "near_sdk::borsh")]
            struct #bitflags_type_ident: u128 {
                #(
                    const #bitflags_idents = 1u128 << #bitflags_idxs;
                )*
            }
        }
    };

    output.into()
}

/// Generates and identifier for the bitflag type that represents permissions.
pub fn new_bitflags_type_ident(span: Span) -> Ident {
    Ident::new(DEFAULT_BITFLAGS_TYPE_NAME, span)
}

fn bitflags_idents(names: &[String], span: Span) -> Vec<Ident> {
    // Assuming enum variant names are in camel case, simply converting them
    // to uppercase is not ideal. However, bitflag identifiers aren't exposed,
    // so let's not bother with converting camel to screaming-snake case.
    let names = names
        .iter()
        .map(|name| name.to_uppercase())
        .collect::<Vec<_>>();
    let admin_names = names
        .iter()
        .map(|name| format!("{name}_ADMIN"))
        .collect::<Vec<_>>();
    let mut idents = vec![Ident::new(DEFAULT_SUPER_ADMIN_NAME, span)];
    for (name, admin_name) in names.iter().zip(admin_names) {
        idents.push(Ident::new(name.as_ref(), span));
        idents.push(Ident::new(admin_name.as_ref(), span));
    }
    idents
}
