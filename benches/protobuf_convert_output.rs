use std::path::Path;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use prost::Message;
use rand::random;
use tailcall::blueprint::GrpcMethod;
use tailcall::grpc::protobuf::ProtobufSet;

pub mod nums {
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/benches/grpc/nums.rs"));
}

const OUT_DIR: &str = "benches/grpc";
const PROTO_FILE: &str = "nums.proto";
const SERVICE_NAME: &str = "nums.NumsService.GetNums";

fn build(proto_file_path: impl AsRef<Path>) -> anyhow::Result<()> {
    std::env::set_var("OUT_DIR", OUT_DIR);

    tonic_build::configure().compile(&[proto_file_path], &["proto"])?;

    Ok(())
}

fn benchmark_convert_output(c: &mut Criterion) {
    let proto_file_path = Path::new(OUT_DIR).join(PROTO_FILE);
    build(&proto_file_path).unwrap();
    let file_descriptor_set = protox::compile([proto_file_path], ["."]).unwrap();
    let protobuf_set = ProtobufSet::from_proto_file(&file_descriptor_set).unwrap();
    let method = GrpcMethod::try_from(SERVICE_NAME).unwrap();
    let service = protobuf_set.find_service(&method).unwrap();
    let protobuf_operation = service.find_operation(&method).unwrap();
    let mut msg: Vec<u8> = vec![0, 0, 0, 0, 14];
    nums::Nums {
        nums: (0..1000).map(|_| random()).collect(),
        flags: (0..1000).map(|_| random()).collect(),
        names: (0..1000)
            .map(|_| (0..100).map(|_| random::<char>()).collect())
            .collect(),
        floats: (0..1000).map(|_| random()).collect(),
    }
    .encode(&mut msg)
    .unwrap();

    c.bench_function("test_batched_body", |b| {
        b.iter(|| {
            black_box(protobuf_operation.convert_output(&msg).unwrap());
        })
    });
}

criterion_group!(benches, benchmark_convert_output);
criterion_main!(benches);
