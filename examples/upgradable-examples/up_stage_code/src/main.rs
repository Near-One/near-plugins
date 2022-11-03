use std::env;
use tokio::runtime::Runtime;
use workspaces::Account;

const WASM_FILEPATH_SECOND: &str =
    "../../target/wasm32-unknown-unknown/release/upgradable_base_second.wasm";

fn main() {
    let args: Vec<String> = env::args().collect();
    let rt = Runtime::new().unwrap();
    let worker = rt.block_on(workspaces::testnet()).unwrap();

    let contract: Account = Account::from_file(args[1].clone(), &worker).unwrap();

    let wasm = std::fs::read(WASM_FILEPATH_SECOND).unwrap();

    println!(
        "{}",
        rt.block_on(
            contract
                .call(contract.id(), "up_stage_code")
                .args_borsh(wasm)
                .max_gas()
                .transact()
        )
        .unwrap()
        .is_success()
    );
}
