use std::{
    io::{self, Error, ErrorKind},
    sync::Arc,
};

use log::info;
use rpcpb::{
    AddPermissionlessValidatorRequest, AddPermissionlessValidatorResponse,
    AddSubnetValidatorsRequest, AddSubnetValidatorsResponse, RemoveSubnetValidatorsRequest,
    RemoveSubnetValidatorsResponse,
};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use tonic::transport::Channel;

pub mod rpcpb {
    tonic::include_proto!("rpcpb");
}
pub use rpcpb::{
    control_service_client::ControlServiceClient, ping_service_client::PingServiceClient,
    AddNodeRequest, AddNodeResponse, BlockchainSpec, HealthRequest, HealthResponse,
    ListBlockchainsRequest, ListBlockchainsResponse, ListSubnetsRequest, ListSubnetsResponse,
    PingRequest, PingResponse, RemoveNodeRequest, RemoveNodeResponse, StartRequest, StartResponse,
    StatusRequest, StatusResponse, StopRequest, StopResponse, UrIsRequest, VmidRequest,
    VmidResponse,
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

    /// Add a node to the currently running cluster.
    pub async fn add_node(&self, req: AddNodeRequest) -> io::Result<AddNodeResponse> {
        let mut control_client = self.grpc_client.control_client.lock().await;
        let req = tonic::Request::new(req);
        let resp = control_client
            .add_node(req)
            .await
            .map_err(|e| Error::new(ErrorKind::Other, format!("failed add_node '{}'", e)))?;

        let add_node_resp = resp.into_inner();
        Ok(add_node_resp)
    }

    /// Remove a node from the currently running cluster.
    pub async fn remove_node(&self, req: RemoveNodeRequest) -> io::Result<RemoveNodeResponse> {
        let mut control_client = self.grpc_client.control_client.lock().await;
        let req = tonic::Request::new(req);
        let resp = control_client
            .remove_node(req)
            .await
            .map_err(|e| Error::new(ErrorKind::Other, format!("failed remove_node '{}'", e)))?;

        let remove_node_resp = resp.into_inner();
        Ok(remove_node_resp)
    }

    /// Adds a new permissionless validator to the network.
    pub async fn add_permissionless_validator(
        &self,
        req: AddPermissionlessValidatorRequest,
    ) -> io::Result<AddPermissionlessValidatorResponse> {
        let mut control_client = self.grpc_client.control_client.lock().await;
        let req = tonic::Request::new(req);
        let resp = control_client
            .add_permissionless_validator(req)
            .await
            .map_err(|e| {
                Error::new(
                    ErrorKind::Other,
                    format!("failed add_permissionless_validator '{}'", e),
                )
            })?;
        let resp = resp.into_inner();
        Ok(resp)
    }

    /// Adds a new permissionless validator to the network.
    pub async fn add_validator(
        &self,
        req: AddSubnetValidatorsRequest,
    ) -> io::Result<AddSubnetValidatorsResponse> {
        let mut control_client = self.grpc_client.control_client.lock().await;
        let req = tonic::Request::new(req);
        let resp = control_client
            .add_subnet_validators(req)
            .await
            .map_err(|e| Error::new(ErrorKind::Other, format!("failed add_validators '{}'", e)))?;
        let resp = resp.into_inner();
        Ok(resp)
    }

    /// Removes a validator from the network.
    pub async fn remove_validator(
        &self,
        req: RemoveSubnetValidatorsRequest,
    ) -> io::Result<RemoveSubnetValidatorsResponse> {
        let mut control_client = self.grpc_client.control_client.lock().await;
        let req = tonic::Request::new(req);
        let resp = control_client
            .remove_subnet_validator(req)
            .await
            .map_err(|e| {
                Error::new(
                    ErrorKind::Other,
                    format!("failed remove_validators '{}'", e),
                )
            })?;
        let resp = resp.into_inner();
        Ok(resp)
    }

    /// Fetches the VM ID for the current cluster.
    pub async fn vm_id(&self, req: VmidRequest) -> io::Result<VmidResponse> {
        let mut control_client = self.grpc_client.control_client.lock().await;
        let req = tonic::Request::new(req);
        let resp = control_client
            .vmid(req)
            .await
            .map_err(|e| Error::new(ErrorKind::Other, format!("failed vm_id '{}'", e)))?;
        let resp = resp.into_inner();
        Ok(resp)
    }

    /// Fetches the list of blockchains for the current cluster.
    pub async fn list_blockchains(
        &self,
        req: ListBlockchainsRequest,
    ) -> io::Result<ListBlockchainsResponse> {
        let mut control_client = self.grpc_client.control_client.lock().await;
        let req = tonic::Request::new(req);
        let resp = control_client.list_blockchains(req).await.map_err(|e| {
            Error::new(ErrorKind::Other, format!("failed list_blockchains '{}'", e))
        })?;
        let resp = resp.into_inner();
        Ok(resp)
    }

    /// Fetches the list of subnets for the current cluster.
    pub async fn list_subnets(&self, req: ListSubnetsRequest) -> io::Result<ListSubnetsResponse> {
        let mut control_client = self.grpc_client.control_client.lock().await;
        let req = tonic::Request::new(req);
        let resp = control_client
            .list_subnets(req)
            .await
            .map_err(|e| Error::new(ErrorKind::Other, format!("failed list_subnets '{}'", e)))?;
        let resp = resp.into_inner();
        Ok(resp)
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
