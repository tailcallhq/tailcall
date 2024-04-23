use std::env::var;
use std::path::PathBuf;

use tailcall_fixtures::get_fixture_path;

fn main() {
    let news = get_fixture_path("grpc/proto/news.proto");

    let out_dir = PathBuf::from(var("OUT_DIR").unwrap());

    tonic_build::configure()
        .file_descriptor_set_path(out_dir.join("news_descriptor.bin"))
        .compile(&[&news], &[news.parent().unwrap()])
        .unwrap();
}
