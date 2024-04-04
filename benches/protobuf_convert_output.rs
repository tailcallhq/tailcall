use std::path::Path;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use prost::Message;
use rand::random;
use tailcall::blueprint::GrpcMethod;
use tailcall::grpc::protobuf::ProtobufSet;

pub mod dummy {
    tonic::include_proto!("dummy");
}

const OUT_DIR: &str = "benches/grpc";
const PROTO_FILE: &str = "dummy.proto";
const SERVICE_NAME: &str = "dummy.DummyService.GetDummy";
const N: usize = 1000;
const M: usize = 100;

pub struct Dummy;

impl Dummy {
    #[allow(clippy::new_ret_no_self)]
    fn new(n: usize, m: usize) -> dummy::Dummy {
        dummy::Dummy {
            ints: (0..n).map(|_| random()).collect(),
            flags: (0..n).map(|_| random()).collect(),
            names: (0..n)
                .map(|_| (0..m).map(|_| random::<char>()).collect())
                .collect(),
            floats: (0..n).map(|_| random()).collect(),
        }
    }
}

fn benchmark_convert_output(c: &mut Criterion) {
    let proto_file_path = Path::new(OUT_DIR).join(PROTO_FILE);
    let file_descriptor_set = protox::compile([proto_file_path], ["."]).unwrap();
    let protobuf_set = ProtobufSet::from_proto_file(file_descriptor_set).unwrap();
    let method = GrpcMethod::try_from(SERVICE_NAME).unwrap();
    let service = protobuf_set.find_service(&method).unwrap();
    let protobuf_operation = service.find_operation(&method).unwrap();
    let mut msg: Vec<u8> = vec![0, 0, 0, 0, 14];
    Dummy::new(N, M).encode(&mut msg).unwrap();

    c.bench_function("test_batched_body", |b| {
        b.iter(|| {
            black_box(
                protobuf_operation
                    .convert_output::<serde_json::Value>(&msg)
                    .unwrap(),
            );
        })
    });
}

criterion_group!(benches, benchmark_convert_output);
criterion_main!(benches);
