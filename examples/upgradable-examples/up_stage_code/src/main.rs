use clap::Parser;
use workspaces::Account;

const DEFAULT_WASM_FILEPATH_SECOND: &str =
    "../../target/wasm32-unknown-unknown/release/upgradable_base_second.wasm";

#[derive(Parser, Default, Debug)]
#[clap(version, about = "Up stage code")]
struct Arguments {
    #[clap(short, long)]
    // Path to key for contract account (for example `$HOME/.near-credentials/testnet/<CONTRACT_ACCOUNT>.json`)
    path_to_key: String,

    #[clap(short, long,  default_value_t = String::from(DEFAULT_WASM_FILEPATH_SECOND))]
    /// Path to wasm file with the new contract
    wasm: String,

    #[clap(long, default_value_t = String::from("testnet"))]
    /// NEAR network (testnet, mainnet, betanet)
    network: String,

    #[clap(short, long)]
    /// Timestamp in nanoseconds to delay deploying the staged code
    deploy_timestamp: Option<u64>,
}

#[macro_export]
macro_rules! get_contract {
    ($network_name:ident, $path_to_key:expr) => {
        Account::from_file($path_to_key, &workspaces::$network_name().await.unwrap()).unwrap()
    };
}

#[tokio::main]
async fn main() {
    let args = Arguments::parse();

    let contract: Account = match &*args.network {
        "testnet" => get_contract!(testnet, args.path_to_key),
        "mainnet" => get_contract!(mainnet, args.path_to_key),
        "betanet" => get_contract!(betanet, args.path_to_key),
        network => panic!(
            "Unknown network {}. Possible networks: testnet, mainnet, betanet",
            network
        ),
    };

    let wasm = std::fs::read(&args.wasm).unwrap();

    println!(
        "{}",
        contract
            .call(contract.id(), "up_stage_code")
            .args_json(serde_json::json!({
                "code": near_sdk::json_types::Base64VecU8(wasm),
                "timestamp": args.deploy_timestamp,
            }))
            .max_gas()
            .transact()
            .await
            .unwrap()
            .is_success()
    );
}
