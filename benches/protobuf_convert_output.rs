use criterion::{black_box, criterion_group, criterion_main, Criterion};
use tailcall::blueprint::GrpcMethod;
use tailcall::grpc::protobuf::ProtobufSet;

const PROTO_FILE_PATH: &str = "src/grpc/tests/proto/greetings.proto";
const SERVICE_NAME: &str = "greetings.Greeter.SayHello";

fn benchmark_convert_output(c: &mut Criterion) {
    let file_descriptor_set = protox::compile([PROTO_FILE_PATH], ["."]).unwrap();
    let protobuf_set = ProtobufSet::from_proto_file(&file_descriptor_set).unwrap();
    let method = GrpcMethod::try_from(SERVICE_NAME).unwrap();
    let service = protobuf_set.find_service(&method).unwrap();
    let protobuf_operation = service.find_operation(&method).unwrap();
    let msg = b"\0\0\0\0\x0e\n\x0ctest message";

    c.bench_function("test_batched_body", |b| {
        b.iter(|| {
            black_box(protobuf_operation.convert_output(msg).unwrap());
        })
    });
}

criterion_group!(benches, benchmark_convert_output);
criterion_main!(benches);
