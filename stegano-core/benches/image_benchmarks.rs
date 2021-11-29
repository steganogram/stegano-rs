use criterion::{criterion_group, criterion_main, Criterion};
use std::io::Read;
use stegano_core::media::image::decoder::ImagePngSource;
use stegano_core::media::image::LsbCodec;
use stegano_core::universal_decoder::{Decoder, OneBitUnveil};

pub fn stegano_image_benchmark(c: &mut Criterion) {
    c.bench_function("Image Decoding", |b| {
        let img = image::open("../resources/with_text/hello_world.png")
            .expect("Input image is not readable.")
            .to_rgba8();

        b.iter(|| {
            let mut buf = vec![0; 13];
            Decoder::new(ImagePngSource::new(&img), OneBitUnveil)
                .read_exact(&mut buf)
                .expect("Failed to read 13 bytes");
            let msg = String::from_utf8(buf).expect("Failed to convert result to string");
            assert_eq!("\u{1}Hello World!", msg)
        })
    });

    c.bench_function("Image Encoding to memory", |b| {
        let plain_image = image::open("../resources/plain/carrier-image.png")
            .expect("Input image is not readable.")
            .to_rgba8();
        let (width, height) = plain_image.dimensions();
        let secret_message = b"Hello World!";

        b.iter(|| {
            let mut image_with_secret = image::RgbaImage::new(width, height);
            LsbCodec::encoder(&mut image_with_secret)
                .write_all(&secret_message[..])
                .expect("Cannot write to codec");
        })
    });
}

criterion_group!(benches, stegano_image_benchmark);
criterion_main!(benches);
