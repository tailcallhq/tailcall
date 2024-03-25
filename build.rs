use std::path::Path;

const OUT_DIR: &str = "benches/grpc";
const PROTO_FILE: &str = "dummy.proto";

fn main() {
    std::env::set_var("OUT_DIR", OUT_DIR);
    let path = protoc_bin_vendored::protoc_bin_path().expect("Failed to find protoc binary");
    std::env::set_var("PROTOC", format!("{}", path.display()));
    let proto_file_path = Path::new(OUT_DIR).join(PROTO_FILE);
    tonic_build::configure()
        .compile(&[proto_file_path], &["proto"])
        .unwrap();
}
