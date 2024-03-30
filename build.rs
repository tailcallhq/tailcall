use std::path::Path;

const PROTO_PATH: &str = "benches/grpc/dummy.proto";

fn main() {
    let path = protoc_bin_vendored::protoc_bin_path().expect("Failed to find protoc binary");
    std::env::set_var("PROTOC", format!("{}", path.display()));
    let proto_file_path = Path::new(PROTO_PATH);
    tonic_build::configure()
        .compile(&[proto_file_path], &["proto"])
        .unwrap();
}
