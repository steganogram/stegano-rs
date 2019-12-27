use criterion::{criterion_group, criterion_main, Criterion};
use stegano::ByteReader;
use std::io::Read;

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("Stegano::ByteReader for resources/HelloWorld_no_passwd_v2.x.png", |b| b.iter(|| {
        let mut dec = ByteReader::new("resources/HelloWorld_no_passwd_v2.x.png");
        let mut buf = Vec::new();
        dec.read_to_end(&mut buf).unwrap();
    }));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);