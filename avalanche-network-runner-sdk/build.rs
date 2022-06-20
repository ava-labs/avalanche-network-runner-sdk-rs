/// ref. https://github.com/hyperium/tonic/tree/master/tonic-build
fn main() {
    tonic_build::configure()
        .build_server(false)
        .build_client(true)
        .compile(&["rpcpb/rpc.proto"], &["rpcpb"])
        .unwrap();
}
