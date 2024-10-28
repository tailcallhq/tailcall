use criterion::Criterion;
use tailcall::core::jit::fixtures::JP;

pub fn bench_synth_nested(c: &mut Criterion) {
    c.bench_function("synth_nested", |b| {
        let placeholder: JP<async_graphql::Value> =
            JP::init("{ posts { id title user { id name } } }", None);
        let synth = placeholder.synth();
        b.iter(|| {
            let a: async_graphql::Value = synth.synthesize().unwrap();
            drop(a);
        })
    });
}
pub fn bench_synth_nested_borrow(c: &mut Criterion) {
    c.bench_function("synth_nested_borrow", |b| {
        let placeholder: JP<serde_json_borrow::Value> =
            JP::init("{ posts { id title user { id name } } }", None);
        let synth = placeholder.synth();
        b.iter(|| {
            let a: serde_json_borrow::Value = synth.synthesize().unwrap();
            drop(a);
        })
    });
}
