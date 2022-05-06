/// ref. https://github.com/hyperium/tonic/tree/master/tonic-build
fn main() {
    tonic_build::configure()
        .build_server(false)
        .build_client(true)
        .compile(
            &[
                "googleapis/google/pubsub/v1/pubsub.proto",
                "avalanche-network-runner/rpcpb/rpc.proto",
            ],
            &["googleapis", "avalanche-network-runner/rpcpb"],
        )
        .unwrap();
}
