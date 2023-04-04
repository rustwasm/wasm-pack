#[path = "../tests/all/utils/fixture.rs"]
#[allow(unused)]
mod fixture;

use criterion::{criterion_group, criterion_main, Criterion};
use std::process::Stdio;

pub fn criterion_benchmark(c: &mut Criterion) {
    let mut g = c.benchmark_group("wasm-pack");
    g.warm_up_time(std::time::Duration::from_secs(10));
    let fixture = fixture::dual_license();
    run(&fixture);
    g.bench_function("re-run build without code changes", |b| {
        b.iter(|| run(&fixture))
    });
}

fn run(fixture: &fixture::Fixture) {
    assert!(fixture
        .wasm_pack()
        .arg("build")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .unwrap()
        .success())
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
