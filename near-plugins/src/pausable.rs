pub trait Pausable {
    fn paused_storage_key(&self) -> Vec<u8>;

    fn is_paused(&self, key: String) -> bool;

    fn paused_keys(&self) -> Option<std::collections::HashSet<String>>;

    fn pause(&mut self, key: String);

    fn unpause(&mut self, key: String);
}
