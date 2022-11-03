use std::env;
use workspaces::Account;

const WASM_FILEPATH_SECOND: &str =
    "../../target/wasm32-unknown-unknown/release/upgradable_base_second.wasm";

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let worker = workspaces::testnet().await.unwrap();

    let contract: Account = Account::from_file(args[1].clone(), &worker).unwrap();

    let wasm = std::fs::read(WASM_FILEPATH_SECOND).unwrap();

    println!(
        "{}",
        contract
            .call(contract.id(), "up_stage_code")
            .args_borsh(wasm)
            .max_gas()
            .transact()
            .await
            .unwrap()
            .is_success()
    );
}
