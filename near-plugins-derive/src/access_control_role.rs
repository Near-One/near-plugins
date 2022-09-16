use proc_macro::TokenStream;
use quote::quote;
use std::convert::TryFrom;
use syn::{parse_macro_input, ItemEnum};

pub fn derive_access_control_role(input: TokenStream) -> TokenStream {
    // This derive doesn't take attributes, so no need to use `darling`.
    let input: ItemEnum = parse_macro_input!(input);
    let ItemEnum {
        ident, variants, ..
    } = input;

    let (variant_idxs, variant_items): (Vec<_>, Vec<_>) =
        variants.iter().cloned().enumerate().unzip();
    let variant_idxs = variant_idxs
        .iter()
        .map(|&idx| {
            u8::try_from(idx).expect("The number of variants should be representable by u8")
        })
        .collect::<Vec<_>>();
    let variant_names = variants
        .iter()
        .map(|v| format!("{}", v.ident))
        .collect::<Vec<_>>();

    let output = quote! {
        impl From<#ident> for u8 {
            fn from(value: #ident) -> Self {
                match value {
                    #(
                        #ident::#variant_items => #variant_idxs,
                    )*
                }
            }
        }

        impl ::std::convert::TryFrom<u8> for #ident {
            type Error = &'static str;

            fn try_from(value: u8) -> Result<Self, Self::Error> {
                match value {
                    #(
                        #variant_idxs => Ok(#ident::#variant_items),
                    )*
                    _ => Err("Value does not correspond to a variant"),
                }
            }
        }

        impl From<#ident> for &'static str {
            fn from(value: #ident) -> Self {
                match value {
                    #(
                        #ident::#variant_items => #variant_names,
                    )*
                }
            }
        }

        impl ::std::convert::TryFrom<&str> for #ident {
            type Error = &'static str;

            fn try_from(value: &str) -> Result<#ident, Self::Error> {
                match value {
                    #(
                        #variant_names => Ok(#ident::#variant_items),
                    )*
                    _ => Err("Value does not correspond to a variant"),
                }
            }
        }

        /// Panics if `n` is too large.
        fn safe_leftshift(value: u128, n: u8) -> u128 {
            value
                .checked_shl(n.into())
                .expect("Too many enum variants to be represented by bitflags")
        }

        // TODO explain enum<->bitflag conversion
        impl AccessControlRole for #ident {
            fn acl_super_admin_permission_bitflag() -> u128 {
                safe_leftshift(1, 0)
            }

            fn acl_permission_bitflag(self) -> u128 {
                let n = (u8::from(self) + 1)
                    .checked_mul(2)
                    .expect("Too many enum variants") - 1;
                safe_leftshift(1, n)
            }

            fn acl_admin_permission_bitflag(self) -> u128 {
                let n = (u8::from(self) + 1)
                    .checked_mul(2)
                    .expect("Too many enum variants");
                safe_leftshift(1, n)
            }
        }

        ::bitflags::bitflags! {
            // TODO generate dynamically
            #[derive(BorshDeserialize, BorshSerialize, Default)]
            struct RoleFlags: u128 {
                const __SUPER_ADMIN = 1u128 << 0;
                const LEVEL1 = 1u128 << 1;
                const LEVEL1_ADMIN = 1u128 << 2;
                const LEVEL2 = 1u128 << 3;
                const LEVEL2_ADMIN = 1u128 << 4;
                const LEVEL3 = 1u128 << 5;
                const LEVEL3_ADMIN = 1u128 << 6;
            }
        }
    };

    output.into()
}
