use proc_macro2::Span;
use proc_macro_crate::crate_name;
use std::str::FromStr;
use syn::{FnArg, Ident, ItemFn};

/// Determines if this block of code was [generated by near_bindgen].
/// Heuristic used is to check for #[no_mangle].
/// TODO: How to make this 100% safe. Discuss with near-sdk team
///
/// [generated by near_bindgen]: https://github.com/near/near-sdk-rs/issues/722
pub(crate) fn is_near_bindgen_wrapped_or_marshall(item: &ItemFn) -> bool {
    let condition_1 = {
        let pattern1 = "(target_arch = \"wasm32\")";
        let pattern2 = "(not(target_arch = \"wasm32\"))";

        item.attrs.iter().any(|attr| {
            let seq = attr.tokens.to_string();
            seq == pattern1 || seq == pattern2
        })
    };
    if condition_1 {
        return true;
    }

    // For a struct `Contract`, `#[near-bindgen]` [generates] `ContractExt`. If
    // `item` is in an implementation of `ContractExt`, the span number of its
    // signature differs from the span number of the self token.
    //
    // [generates]: https://github.com/near/near-sdk-rs/pull/742
    let condition_2 = {
        let signature_span = item.sig.ident.span();
        let self_token_span = match item.sig.inputs.iter().nth(0) {
            Some(FnArg::Receiver(receiver)) => receiver.self_token.span,
            _ => panic!("Attribute must be used on a method with self receiver"),
        };
        span_number(&signature_span) != span_number(&self_token_span)
    };
    condition_2
}

/// Returns the number of the span.
///
/// # Panics
///
/// Panics if the formatted `span` does not correspond to the pattern
/// `"#42 bytes(1124..1142)"`.
fn span_number(span: &Span) -> u64 {
    let formatted = format!("{:#?}", span);
    let mut number_part = formatted
        .split(" ")
        .nth(0)
        .expect("Formatting a Span yielded an unexpected pattern")
        .to_string();
    number_part.remove(0); // remove the `#`
    u64::from_str(&number_part).expect("Failed to extract number from formatted Span")
}

pub(crate) fn cratename() -> Ident {
    Ident::new(
        &crate_name("near-plugins").unwrap_or_else(|_| "near_plugins".to_string()),
        Span::call_site(),
    )
}
