use criterion::Criterion;
use tailcall::core::jit::common::JsonPlaceholder;
use tailcall::core::jit::SynthBorrow;

pub fn bench_synth_nested(c: &mut Criterion) {
    c.bench_function("synth_nested", |b| {
        let synth: Box<SynthBorrow> =
            JsonPlaceholder::init("{ posts { id title user { id name } } }");
        b.iter(|| {
            let a = synth.synthesize();
            drop(a);
        })
    });
}
