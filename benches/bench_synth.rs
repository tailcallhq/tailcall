use criterion::Criterion;
use tailcall::core::ir::common::JsonPlaceholder;

pub fn bench_synth_nested(c: &mut Criterion) {
    c.bench_function("synth_nested", |b| {
        let synth = JsonPlaceholder::init("{ posts { id title user { id name } } }");
        insta::assert_snapshot!(synth.synthesize());
        b.iter(|| {
            let a = synth.synthesize();
            drop(a);
        })
    });
}
