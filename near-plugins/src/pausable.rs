/// # Pausable
///
/// Trait which allows contracts to implement an emergency stop mechanism that can be triggered
/// by an authorized account. This authorized account can pause certain features which will
/// prevent some methods or behaviors to be executed. It is expected as well that some methods
/// only work in case certain feature is paused, this will be useful to implement escape hatches.
///
/// Features are identified by keys.
///
/// ## Default implementatino:
///
/// Key "ALL" is understood to pause all "pausable" features at once.
/// Provided implementation is optimized for the case where only a small amount of features are
/// paused at a single moment. If all features are meant to be paused, use "ALL" instead. This is done
/// by storing all paused keys in a single slot on the storage.
///
/// Only owner and self can call `pa_pause_feature` / `pa_unpause_feature`. Requires the contract to
/// be Ownable.
///
/// ## Credits:
///
/// Inspired by Open Zeppelin Pausable module:
/// https://github.com/OpenZeppelin/openzeppelin-contracts/blob/master/contracts/security/Pausable.sol
use near_sdk::AccountId;

pub trait Pausable {
    /// Key of storage slot with list of paused features.
    /// By default b"__PAUSED__" is used.
    fn pa_storage_key(&self) -> Vec<u8>;

    /// Check if a feature is paused
    fn pa_is_paused(&self, key: String) -> bool;

    /// List of all current paused features
    fn pa_all_paused(&self) -> Option<Vec<String>>;

    /// Pause specified feature.
    fn pa_pause_feature(&mut self, key: String);

    /// Unpause specified feature
    fn pa_unpause_features(&mut self, key: String);
}

/// Event emitted when a feature is paused.
struct Pause {
    /// Account Id that triggered the pause.
    by: AccountId,
    /// Key identifying the feature that was paused.
    key: String,
}

/// Event emitted when a feature is unpaused.
struct Unpause {
    /// Account Id that triggered the unpause.
    by: AccountId,
    /// Key identifying the feature that was unpaused.
    key: String,
}

/// TODO: Macro that only runs when some functionality is paused #[on_pause("features", not_all)]
