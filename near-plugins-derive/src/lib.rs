#![allow(
    clippy::cognitive_complexity,
    clippy::collection_is_never_read,
    clippy::too_many_lines
)]
use proc_macro::{self, TokenStream};

mod access_control_role;
mod access_controllable;
mod ownable;
mod pausable;
mod upgradable;
mod utils;

/// Defines the derive macro for `Ownable`.
#[proc_macro_derive(Ownable, attributes(ownable))]
pub fn derive_ownable(input: TokenStream) -> TokenStream {
    ownable::derive_ownable(input)
}

/// Defines attribute macro `only`.
#[proc_macro_attribute]
pub fn only(attrs: TokenStream, item: TokenStream) -> TokenStream {
    ownable::only(attrs, item)
}

/// Defines the derive macro for `Upgradable`.
#[proc_macro_derive(Upgradable, attributes(upgradable))]
pub fn derive_upgradable(input: TokenStream) -> TokenStream {
    upgradable::derive_upgradable(input)
}

/// Defines the derive macro for `Pausable`.
#[proc_macro_derive(Pausable, attributes(pausable))]
pub fn derive_pausable(input: TokenStream) -> TokenStream {
    pausable::derive_pausable(input)
}

/// Defines the attribute macro `pause`.
#[proc_macro_attribute]
pub fn pause(attrs: TokenStream, item: TokenStream) -> TokenStream {
    pausable::pause(attrs, item)
}

/// Defines the attribute macro `if_paused`.
#[proc_macro_attribute]
pub fn if_paused(attrs: TokenStream, item: TokenStream) -> TokenStream {
    pausable::if_paused(attrs, item)
}

/// Defines the derive macro for `AccessControlRole`.
#[proc_macro_derive(AccessControlRole)]
pub fn derive_access_control_role(input: TokenStream) -> TokenStream {
    access_control_role::derive_access_control_role(input)
}

/// Defines the attribute macro `access_control`.
#[proc_macro_attribute]
pub fn access_control(attrs: TokenStream, item: TokenStream) -> TokenStream {
    access_controllable::access_controllable(attrs, item)
}

/// Defines the attribute macro `access_control_any`.
#[proc_macro_attribute]
pub fn access_control_any(attrs: TokenStream, item: TokenStream) -> TokenStream {
    access_controllable::access_control_any(attrs, item)
}
