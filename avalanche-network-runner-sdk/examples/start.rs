use std::env::args;

use log::info;
use tokio::runtime::Runtime;

use avalanche_network_runner_sdk::{Client, GlobalConfig};
use avalanche_proto::rpcpb::StartRequest;

/// cargo run --example start -- [HTTP RPC ENDPOINT] [EXEC PATH]
/// cargo run --example start -- http://127.0.0.1:8080 ~/Documents/avalanchego/build/avalanchego
fn main() {
    // ref. https://github.com/env-logger-rs/env_logger/issues/47
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );

    let url = args().nth(1).expect("no url given");
    let exec_path = args().nth(2).expect("no exec path given");
    let rt = Runtime::new().unwrap();

    info!("creating client");
    let cli = rt.block_on(Client::new(&url));
    let global_config = GlobalConfig {
        log_level: String::from("info"),
    };

    let resp = rt
        .block_on(cli.start(StartRequest {
            exec_path,
            num_nodes: Some(5),
            global_node_config: Some(serde_json::to_string(&global_config).unwrap()),
            ..Default::default()
        }))
        .expect("failed start");
    info!("start response: {:?}", resp);
}
