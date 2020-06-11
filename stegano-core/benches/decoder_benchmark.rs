use criterion::{criterion_group, criterion_main, Criterion};
use std::io::Read;
use stegano_core::LSBCodec;

pub fn criterion_benchmark(c: &mut Criterion) {
    let mut img = image::open("../resources/with_text/hello_world.png")
        .expect("Input image is not readable.")
        .to_rgba();

    c.bench_function(
        "SteganoCore::LSBCodec for resources/with_text/hello_world.png (decode)",
        |b| {
            b.iter(|| {
                let mut dec = LSBCodec::new(&mut img);
                let mut buf = vec![0; 13];
                dec.read_exact(&mut buf).expect("Failed to read 13 bytes");
                let msg = String::from_utf8(buf).expect("Failed to convert result to string");
                assert_eq!("\u{1}Hello World!", msg)
            })
        },
    );
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
