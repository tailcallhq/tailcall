use std::path::Path;

const OUT_DIR: &str = "benches/grpc";
const PROTO_FILE: &str = "dummy.proto";

fn main() {
    std::env::set_var("OUT_DIR", OUT_DIR);
    let proto_file_path = Path::new(OUT_DIR).join(PROTO_FILE);
    tonic_build::configure()
        .compile(&[proto_file_path], &["proto"])
        .unwrap();
}
