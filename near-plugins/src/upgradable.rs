/// Upgradable implementation inspired by [NEP123](https://github.com/near/NEPs/pull/123).
/// There is no timer or staging duration implemented by default.
///
/// To upgrade the contract, first the code needs to be staged, and then it can be deployed.
/// Default implementation allows only owner or self to stage and deploy.
use near_sdk;

pub trait Upgradable {
    fn code_storage_key(&self) -> Vec<u8>;

    fn stage_code(&mut self, code: Vec<u8>);

    fn get_staged_code(&self) -> Option<Vec<u8>>;

    fn get_staged_code_hash(&self) -> Option<String>;

    fn deploy_code(&mut self) -> near_sdk::Promise;
}
