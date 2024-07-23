use criterion::Criterion;
use tailcall::core::jit::common::JsonPlaceholder;

pub fn bench_synth_nested(c: &mut Criterion) {
    c.bench_function("synth_nested", |b| {
        let placeholder = JsonPlaceholder::init("{ posts { id title user { id name } } }");
        let synth = placeholder.synth();
        b.iter(|| {
            let a: async_graphql::Value = synth.synthesize().unwrap();
            drop(a);
        })
    });
}
