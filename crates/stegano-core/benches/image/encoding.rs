use criterion::{criterion_group, criterion_main, Criterion};
use std::io::Write;
use stegano_core::media::image::encoder::ImageRgbaColorMut;
use stegano_core::universal_encoder::{Encoder, OneBitHide};

pub fn image_encoding(c: &mut Criterion) {
    c.bench_function("Image Encoding", |b| {
        let mut plain_image = image::open("tests/images/plain/carrier-image.png")
            .expect("Input image is not readable.")
            .to_rgba8();
        let secret_message = b"Hello World!";

        b.iter(|| {
            Encoder::new(ImageRgbaColorMut::new(&mut plain_image), OneBitHide)
                .write_all(&secret_message[..])
                .expect("Cannot write secret message");
        })
    });
}

criterion_group!(benches, image_encoding);
criterion_main!(benches);
