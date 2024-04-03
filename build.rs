const PROTO_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/benches/grpc/dummy.proto");

fn main() {
    let path = protoc_bin_vendored::protoc_bin_path().expect("Failed to find protoc binary");
    std::env::set_var("PROTOC", format!("{}", path.display()));
    tonic_build::compile_protos(PROTO_PATH).unwrap();
}
