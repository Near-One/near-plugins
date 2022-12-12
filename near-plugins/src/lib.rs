pub mod access_control_role;
pub mod access_controllable;
pub mod events;
pub mod full_access_key_fallback;
pub mod ownable;
pub mod pausable;
#[cfg(not(target_arch = "wasm32"))]
mod test_utils;
pub mod upgradable;

pub use access_control_role::AccessControlRole;
pub use access_controllable::AccessControllable;
pub use full_access_key_fallback::FullAccessKeyFallback;
pub use upgradable::UpgradableDurationStatus;
pub use near_plugins_derive::{
    access_control, access_control_any, if_paused, only, pause, AccessControlRole,
    FullAccessKeyFallback, Ownable, Pausable, Upgradable,
};
pub use ownable::Ownable;
pub use pausable::Pausable;
pub use upgradable::Upgradable;

// Re-exporting these dependencies avoids requiring contracts to depend on them.
// For example, without re-exporting `bitflags` a contract using the access
// control plugin would require:
//
// ```toml
// # Cargo.toml
// [dependencies]
// bitflags = "x.y"
// ```
#[doc(hidden)]
pub use bitflags;
