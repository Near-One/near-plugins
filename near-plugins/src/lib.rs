pub mod events;
pub mod full_access_key_fallback;
pub mod ownable;
pub mod pausable;
#[cfg(not(target_arch = "wasm32"))]
mod test_utils;
pub mod upgradable;

pub use full_access_key_fallback::FullAccessKeyFallback;
pub use near_plugins_derive::{
    if_paused, only, pause, FullAccessKeyFallback, Ownable, Pausable, Upgradable,
};
pub use ownable::Ownable;
pub use pausable::Pausable;
pub use upgradable::Upgradable;
