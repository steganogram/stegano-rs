use criterion::{criterion_group, criterion_main, Criterion};
use stegano::ByteReader;
use std::io::Read;

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("Stegano::ByteReader for resources/with_text/hello_world.png", |b| b.iter(|| {
        let mut dec = ByteReader::new("resources/with_text/hello_world.png");
        let mut buf = Vec::new();
        dec.read_to_end(&mut buf).unwrap();
    }));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);