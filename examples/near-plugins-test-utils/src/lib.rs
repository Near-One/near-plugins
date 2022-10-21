use serde_json::json;
use near_sdk::ONE_NEAR;
use workspaces::{Account, Contract};

pub async fn get_contract(wasm_path: &str) -> (Account, Contract) {
    let worker = workspaces::sandbox().await.unwrap();

    let owner = worker.root_account().unwrap();

    let wasm = std::fs::read(wasm_path).unwrap();
    let contract = owner.deploy(&wasm).await.unwrap().unwrap();

    (owner, contract)
}

pub async fn view(contract: &Contract, method_name: &str, args: &serde_json::Value) -> Vec<u8> {
    contract.view(method_name,
                      args.to_string().into_bytes()).await.unwrap().result
}

pub async fn call_arg(contract: &Contract, method_name: &str, args: &serde_json::Value) -> bool {
    contract
        .call(method_name)
        .args_json(args)
        .max_gas()
        .transact()
        .await
        .unwrap()
        .is_success()
}

pub async fn call_by_with_arg(account: &Account, contract: &Contract, method_name: &str, args: &serde_json::Value) -> bool {
    account.call(contract.id(), method_name)
        .args_json(args)
        .max_gas()
        .transact()
        .await.unwrap().is_success()
}

pub async fn get_subaccount(account: &Account, new_account_name: &str) -> Account {
    account
        .create_subaccount(new_account_name)
        .initial_balance(ONE_NEAR)
        .transact()
        .await
        .unwrap()
        .unwrap()
}

pub async fn get_contract_testnet(wasm_file: &str) -> (Account, Contract) {
    let worker = workspaces::testnet().await.unwrap();

    let wasm = std::fs::read(wasm_file).unwrap();
    let contract: Contract = worker.dev_deploy(&wasm).await.unwrap();

    let owner = contract.as_account();

    (owner.clone(), contract)
}

#[macro_export]
macro_rules! view {
    ($contract:ident, $method_name:literal) => {
        serde_json::from_slice(&view(&$contract, $method_name, &json!({})).await).unwrap()
    };
    ($contract:ident, $method_name:literal, $args:expr) => {
        serde_json::from_slice(&view(&$contract, $method_name, $args).await).unwrap()
    };
}

#[macro_export]
macro_rules! call {
    ($contract:ident, $method_name:literal) => {
        call_arg(&$contract, $method_name, &json!({}))
    };
    ($contract:ident, $method_name:literal, $args:expr) => {
        call_arg(&$contract, $method_name, $args)
    };
    ($account:expr, $contract:ident, $method_name:literal) => {
        call_by_with_arg($account, &$contract, $method_name, &json!({}))
    };
    ($account:expr, $contract:ident, $method_name:literal, $args:expr) => {
        call_by_with_arg($account, &$contract, $method_name, $args)
    };
}

pub async fn check_counter(contract: &Contract, expect_counter: u64) {
    let counter: u64 = view!(contract, "get_counter");
    assert_eq!(counter, expect_counter);
}