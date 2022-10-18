use near_sdk::ONE_NEAR;
use tokio::runtime::Runtime;
use workspaces::{Account, Contract};

pub fn get_contract(wasm_path: &str) -> (Account, Contract) {
    let rt = Runtime::new().unwrap();
    let worker = rt.block_on(workspaces::sandbox()).unwrap();

    let owner = worker.root_account().unwrap();

    let wasm = std::fs::read(wasm_path).unwrap();
    let contract = rt.block_on(owner.deploy(&wasm)).unwrap().unwrap();

    (owner, contract)
}

pub fn view(contract: &Contract, method_name: &str, args: &serde_json::Value) -> Vec<u8> {
    let rt = Runtime::new().unwrap();

    rt.block_on(
        contract.view(method_name,
                      args.to_string().into_bytes())
    ).unwrap().result
}

pub fn call(contract: &Contract, method_name: &str) -> bool {
    let rt = Runtime::new().unwrap();

    rt.block_on(contract.call(method_name).max_gas().transact())
        .unwrap()
        .is_success()
}

pub fn call_arg(contract: &Contract, method_name: &str, args: &serde_json::Value) -> bool {
    let rt = Runtime::new().unwrap();

    rt.block_on(
        contract
            .call(method_name)
            .args_json(args)
            .max_gas()
            .transact(),
    )
    .unwrap()
    .is_success()
}

pub fn call_by(account: &Account, contract: &Contract, method_name: &str) -> bool {
    let rt = Runtime::new().unwrap();

    rt.block_on(
        account
            .call(contract.id(), method_name)
            .max_gas()
            .transact(),
    )
    .unwrap()
    .is_success()
}

pub fn call_by_with_arg(account: &Account, contract: &Contract, method_name: &str, args: &serde_json::Value) -> bool {
    let rt = Runtime::new().unwrap();

    rt.block_on(
        account.call(contract.id(), method_name)
            .args_json(args)
            .max_gas()
            .transact()
    ).unwrap().is_success()
}

pub fn get_subaccount(account: &Account, new_account_name: &str) -> Account {
    let rt = Runtime::new().unwrap();

    rt.block_on(
        account
            .create_subaccount(new_account_name)
            .initial_balance(ONE_NEAR)
            .transact(),
    )
    .unwrap()
    .unwrap()
}

pub fn get_contract_testnet(wasm_file: &str) -> (Account, Contract) {
    let rt = Runtime::new().unwrap();
    let worker = rt.block_on(workspaces::testnet()).unwrap();

    let wasm = std::fs::read(wasm_file).unwrap();
    let contract: Contract = rt.block_on(worker.dev_deploy(&wasm)).unwrap();

    let owner = contract.as_account();

    (owner.clone(), contract)
}

#[macro_export]
macro_rules! view {
    ($contract:ident, $method_name:literal) => {
        serde_json::from_slice(&view(&$contract, $method_name, &json!({}))).unwrap()
    };
    ($contract:ident, $method_name:literal, $args:expr) => {
            serde_json::from_slice(&view(&$contract, $method_name, $args)).unwrap()
    };
}
