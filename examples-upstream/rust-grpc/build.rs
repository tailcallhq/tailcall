use std::path::PathBuf;

fn main() {
    let mut news = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    news.push("news.proto");
    tonic_build::compile_protos(news).expect("Failed to compile protos");

    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());

    tonic_build::configure()
        .file_descriptor_set_path(out_dir.join("news_descriptor.bin"))
        .compile(&["news.proto"], &["proto"])
        .unwrap();
}
