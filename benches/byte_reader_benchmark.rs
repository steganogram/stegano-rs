use criterion::{criterion_group, criterion_main, Criterion};
use stegano::LSBCodec;
use std::io::Read;

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("Stegano::LSBCodec for resources/with_text/hello_world.png (decode)", |b| b.iter(|| {
        let mut img = image::open("resources/with_text/hello_world.png")
            .expect("Input image is not readable.")
            .to_rgba();

        let mut dec = LSBCodec::new(&mut img);
        let mut buf = Vec::new();
        dec.read_to_end(&mut buf).unwrap();
    }));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);