use criterion::{criterion_group, criterion_main, Criterion};
use std::io::Read;
use stegano_core::media::image::decoder::ImageRgbaColor;
use stegano_core::universal_decoder::{Decoder, OneBitUnveil};

pub fn image_decoding(c: &mut Criterion) {
    c.bench_function("Image Decoding", |b| {
        let img = image::open("tests/images/with_text/hello_world.png")
            .expect("Input image is not readable.")
            .to_rgba8();
        let mut buf = [0; 13];

        b.iter(|| {
            Decoder::new(ImageRgbaColor::new(&img), OneBitUnveil)
                .read_exact(&mut buf)
                .expect("Failed to read 13 bytes");
        })
    });
}

criterion_group!(benches, image_decoding);
criterion_main!(benches);
