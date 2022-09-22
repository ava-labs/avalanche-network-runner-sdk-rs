use std::{
    io::{self, Error, ErrorKind},
    sync::Arc,
};

use log::info;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use tonic::transport::Channel;

pub mod rpcpb {
    tonic::include_proto!("rpcpb");
}
pub use rpcpb::{
    control_service_client::ControlServiceClient, ping_service_client::PingServiceClient,
    HealthRequest, HealthResponse, PingRequest, PingResponse, StartRequest, StartResponse,
    StatusRequest, StatusResponse, StopRequest, StopResponse, UrIsRequest,
};

pub struct Client<T> {
    pub rpc_endpoint: String,
    /// Shared gRPC client connections.
    pub grpc_client: Arc<GrpcClient<T>>,
}

pub struct GrpcClient<T> {
    pub ping_client: Mutex<PingServiceClient<T>>,
    pub control_client: Mutex<ControlServiceClient<T>>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct GlobalConfig {
    pub log_level: String,
}

#[derive(Serialize, Deserialize)]
pub struct BlockchainSpecs {
    #[serde(rename = "vm_name")]
    /// Name of the Vm.
    pub vm_name: String,

    /// Path to genesis file.
    pub genesis: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "subnet_id")]
    /// Id for the subnet.
    pub subnet_id: Option<String>,
}

impl Client<Channel> {
    /// Creates a new network-runner client.
    ///
    /// # Arguments
    ///
    /// * `rpc_endpoint` - HTTP RPC endpoint to the network runner server.
    pub async fn new(rpc_endpoint: &str) -> Self {
        info!("creating a new client with {}", rpc_endpoint);
        let ep = String::from(rpc_endpoint);
        let ping_client = PingServiceClient::connect(ep.clone()).await.unwrap();
        let control_client = ControlServiceClient::connect(ep).await.unwrap();
        let grpc_client = GrpcClient {
            ping_client: Mutex::new(ping_client),
            control_client: Mutex::new(control_client),
        };
        Self {
            rpc_endpoint: String::from(rpc_endpoint),
            grpc_client: Arc::new(grpc_client),
        }
    }

    /// Pings the network-runner server.
    pub async fn ping(&self) -> io::Result<PingResponse> {
        let mut ping_client = self.grpc_client.ping_client.lock().await;
        let req = tonic::Request::new(PingRequest {});
        let resp = ping_client
            .ping(req)
            .await
            .map_err(|e| Error::new(ErrorKind::Other, format!("failed ping '{}'", e)))?;

        let ping_resp = resp.into_inner();
        Ok(ping_resp)
    }

    /// Starts a cluster.
    pub async fn start(&self, req: StartRequest) -> io::Result<StartResponse> {
        let mut control_client = self.grpc_client.control_client.lock().await;
        let req = tonic::Request::new(req);
        let resp = control_client
            .start(req)
            .await
            .map_err(|e| Error::new(ErrorKind::Other, format!("failed stop '{}'", e)))?;

        let start_resp = resp.into_inner();
        Ok(start_resp)
    }

    /// Fetches the current cluster health information via network-runner.
    pub async fn health(&self) -> io::Result<HealthResponse> {
        let mut control_client = self.grpc_client.control_client.lock().await;
        let req = tonic::Request::new(HealthRequest {});
        let resp = control_client
            .health(req)
            .await
            .map_err(|e| Error::new(ErrorKind::Other, format!("failed status '{}'", e)))?;

        let health_resp = resp.into_inner();
        Ok(health_resp)
    }

    /// Fetches the URIs for the current cluster.
    pub async fn uris(&self) -> io::Result<Vec<String>> {
        let mut control_client = self.grpc_client.control_client.lock().await;
        let req = tonic::Request::new(UrIsRequest {});
        let resp = control_client
            .ur_is(req)
            .await
            .map_err(|e| Error::new(ErrorKind::Other, format!("failed status '{}'", e)))?;

        let uris_resp = resp.into_inner();
        Ok(uris_resp.uris)
    }

    /// Fetches the current cluster status via network-runner.
    pub async fn status(&self) -> io::Result<StatusResponse> {
        let mut control_client = self.grpc_client.control_client.lock().await;
        let req = tonic::Request::new(StatusRequest {});
        let resp = control_client
            .status(req)
            .await
            .map_err(|e| Error::new(ErrorKind::Other, format!("failed status '{}'", e)))?;

        let status_resp = resp.into_inner();
        Ok(status_resp)
    }

    /// Stop the currently running cluster.
    pub async fn stop(&self) -> io::Result<StopResponse> {
        let mut control_client = self.grpc_client.control_client.lock().await;
        let req = tonic::Request::new(StopRequest {});
        let resp = control_client
            .stop(req)
            .await
            .map_err(|e| Error::new(ErrorKind::Other, format!("failed stop '{}'", e)))?;

        let stop_resp = resp.into_inner();
        Ok(stop_resp)
    }
}

#[test]
fn global_config() {
    use assert_json_diff::assert_json_include;
    use serde_json::json;

    let conf = GlobalConfig {
        log_level: "debug".to_string(),
    };
    let expected = json!({
        "log-level": "debug",
    });

    let actual = serde_json::to_value(conf).unwrap();
    assert_json_include!(actual: actual, expected: expected)
}

#[test]
fn block_chain_spec() {
    use assert_json_diff::assert_json_include;
    use serde_json::json;

    let spec = BlockchainSpecs {
        vm_name: "minikvvm".to_string(),
        genesis: "/tmp/genesis.json".to_string(),
        subnet_id: None,
    };

    let expected = json!({
        "vm_name": "minikvvm",
        "genesis": "/tmp/genesis.json"
    });

    let actual = serde_json::to_value(spec).unwrap();
    assert_json_include!(actual: actual, expected: expected);

    let spec = BlockchainSpecs {
        vm_name: "minikvvm".to_string(),
        genesis: "/tmp/genesis.json".to_string(),
        subnet_id: Some("qBnAKUQ2mxiB1JdqsPPU7Ufuj1XmPLpnPTRvZEpkYZBmK6UjE".to_string()),
    };

    let expected = json!({
        "vm_name": "minikvvm",
        "genesis": "/tmp/genesis.json",
        "subnet_id": "qBnAKUQ2mxiB1JdqsPPU7Ufuj1XmPLpnPTRvZEpkYZBmK6UjE",
    });

    let actual = serde_json::to_value(spec).unwrap();
    assert_json_include!(actual: actual, expected: expected);
}
