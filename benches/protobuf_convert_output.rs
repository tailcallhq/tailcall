use std::path::Path;

use anyhow::Result;
use criterion::{black_box, Criterion};
use rand::{thread_rng, Fill};
use serde_json::{json, Value};
use tailcall::core::blueprint::GrpcMethod;
use tailcall::core::grpc::protobuf::ProtobufSet;

const PROTO_DIR: &str = "benches/grpc";
const PROTO_FILE: &str = "dummy.proto";
const SERVICE_NAME: &str = "dummy.DummyService.GetDummy";
const N: usize = 1000;
const M: usize = 100;

fn create_dummy_value(n: usize, m: usize) -> Result<Value> {
    let rng = &mut thread_rng();
    let mut ints = vec![0i32; n];
    let mut floats = vec![0f32; n];
    let mut flags = vec![false; n];
    let names: Vec<String> = (0..n)
        .map(|_| {
            let mut chars = vec![' '; m];

            chars.try_fill(rng)?;

            Ok(chars.into_iter().collect::<String>())
        })
        .collect::<Result<_>>()?;

    ints.try_fill(rng)?;
    floats.try_fill(rng)?;
    flags.try_fill(rng)?;

    let value = json!({
        "ints": ints,
        "floats": floats,
        "flags": flags,
        "names": names,
    });

    Ok(value)
}

pub fn benchmark_convert_output(c: &mut Criterion) {
    let proto_file_path = Path::new(PROTO_DIR).join(PROTO_FILE);
    let file_descriptor_set = protox::compile([proto_file_path], ["."]).unwrap();
    let protobuf_set = ProtobufSet::from_proto_file(file_descriptor_set).unwrap();
    let method = GrpcMethod::try_from(SERVICE_NAME).unwrap();
    let service = protobuf_set.find_service(&method).unwrap();
    let protobuf_operation = service.find_operation(&method).unwrap();

    let dummy_value = create_dummy_value(N, M).unwrap();
    let msg = protobuf_operation
        .convert_input(&dummy_value.to_string())
        .unwrap();

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
