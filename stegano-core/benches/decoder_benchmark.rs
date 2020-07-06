use criterion::{criterion_group, criterion_main, Criterion};
use std::io::Read;
use stegano_core::carriers::image::decoder::ImagePngSource;
use stegano_core::universal_decoder::{Decoder, OneBitUnveil};

pub fn stegano_image_benchmark(c: &mut Criterion) {
    let mut img = image::open("../resources/with_text/hello_world.png")
        .expect("Input image is not readable.")
        .to_rgba();

    c.bench_function("SteganoCore Image Decoding", |b| {
        b.iter(|| {
            let mut buf = vec![0; 13];
            Decoder::new(ImagePngSource::new(&mut img), OneBitUnveil)
                .read_exact(&mut buf)
                .expect("Failed to read 13 bytes");
            let msg = String::from_utf8(buf).expect("Failed to convert result to string");
            assert_eq!("\u{1}Hello World!", msg)
        })
    });
}

pub fn stegano_audio_benchmark(c: &mut Criterion) {
    use hound::{WavReader, WavWriter};
    use std::path::Path;
    use stegano_core::carriers::audio::LSBCodec;
    use tempdir::TempDir;

    let audio_with_secret: &Path = "../resources/secrets/audio-with-secrets.wav".as_ref();
    c.bench_function("SteganoCore Audio Decoding", |b| {
        b.iter(|| {
            let mut reader = WavReader::open(audio_with_secret).expect("Cannot create reader");
            let mut buf = vec![0; 12];
            LSBCodec::decoder(&mut reader)
                .read_exact(&mut buf[..])
                .expect("Cannot read 12 bytes from codec");
            let msg = String::from_utf8(buf).expect("Cannot convert result to string");
            assert_eq!("Hello World!", msg);
        })
    });
}

criterion_group!(benches, stegano_image_benchmark, stegano_audio_benchmark);
criterion_main!(benches);
