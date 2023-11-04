use iai_callgrind::{black_box, main, library_benchmark_group, library_benchmark};

fn fibonacci(n: u64) -> u64 {
    match n {
        0 => 1,
        1 => 1,
        n => fibonacci(n - 1) + fibonacci(n - 2),
    }
}

#[library_benchmark]
#[bench::short(10)]
#[bench::long(30)]
fn bench_fibonacci(value: u64) -> u64 {
    black_box(fibonacci(value))
}


library_benchmark_group!(
    name = bench_fibonacci_group;
    benchmarks = bench_fibonacci
);

main!(library_benchmark_groups = bench_fibonacci_group);
