use std::{
    thread,
    time::{Duration, Instant},
};

use log::{info, warn};

use avalanche_network_runner_sdk::{Client, StartRequest};

#[tokio::test]
async fn e2e() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .is_test(true)
        .try_init();

    let (ep, is_set) = get_network_runner_grpc_endpoint();
    assert!(is_set);

    let cli = Client::new(&ep).await;

    info!("ping...");
    let resp = cli.ping().await.expect("failed ping");
    info!("network-runner is running (ping response {:?})", resp);

    let (exec_path, is_set) = get_network_runner_avalanchego_path();
    assert!(is_set);

    info!("starting...");
    let resp = cli
        .start(StartRequest {
            exec_path,
            log_level: Some(String::from("INFO")),
            ..Default::default()
        })
        .await
        .expect("failed start");
    info!(
        "started avalanchego cluster with network-runner: {:?}",
        resp
    );

    // enough time for network-runner to get ready
    thread::sleep(Duration::from_secs(20));

    info!("checking cluster healthiness...");
    let mut ready = false;
    let timeout = Duration::from_secs(300);
    let interval = Duration::from_secs(15);
    let start = Instant::now();
    let mut cnt: u128 = 0;
    loop {
        let elapsed = start.elapsed();
        if elapsed.gt(&timeout) {
            break;
        }

        let itv = {
            if cnt == 0 {
                // first poll with no wait
                Duration::from_secs(1)
            } else {
                interval
            }
        };
        thread::sleep(itv);

        ready = {
            match cli.health().await {
                Ok(_) => {
                    info!("healthy now!");
                    true
                }
                Err(e) => {
                    warn!("not healthy yet {}", e);
                    false
                }
            }
        };
        if ready {
            break;
        }

        cnt += 1;
    }
    assert!(ready);

    info!("checking status...");
    let status = cli.status().await.expect("failed status");
    assert!(status.cluster_info.is_some());
    let cluster_info = status.cluster_info.unwrap();
    let mut rpc_eps: Vec<String> = Vec::new();
    for (node_name, iv) in cluster_info.node_infos.into_iter() {
        info!("{}: {}", node_name, iv.uri);
        rpc_eps.push(iv.uri.clone());
    }
    info!("avalanchego RPC endpoints: {:?}", rpc_eps);

    // TODO: do some tests...

    info!("stopping...");
    let _resp = cli.stop().await.expect("failed stop");
    info!("successfully stopped network");
}

fn get_network_runner_grpc_endpoint() -> (String, bool) {
    match std::env::var("NETWORK_RUNNER_GRPC_ENDPOINT") {
        Ok(s) => (s, true),
        _ => (String::new(), false),
    }
}

fn get_network_runner_avalanchego_path() -> (String, bool) {
    match std::env::var("NETWORK_RUNNER_AVALANCHEGO_PATH") {
        Ok(s) => (s, true),
        _ => (String::new(), false),
    }
}
