use proc_macro2::Span;
use proc_macro_crate::crate_name;
use syn::{Ident, ItemFn};

/// Determines if this block of code was generated by near_bindgen.
/// Heuristic used is to check for #[no_mangle].
/// TODO: How to make this 100% safe. Discuss with near-sdk team
pub(crate) fn is_near_bindgen_wrapped_or_marshall(item: &ItemFn) -> bool {
    let pattern1 = "(target_arch = \"wasm32\")";
    let pattern2 = "(not(target_arch = \"wasm32\"))";

    item.attrs.iter().any(|attr| {
        let seq = attr.tokens.to_string();
        seq == pattern1 || seq == pattern2
    })
}

pub(crate) fn cratename() -> Ident {
    Ident::new(
        &crate_name("near-plugins").unwrap_or_else(|_| "near_plugins".to_string()),
        Span::call_site(),
    )
}
