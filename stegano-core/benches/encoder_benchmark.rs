use criterion::{criterion_group, criterion_main, Criterion};
use stegano_core::carriers::image::LSBCodec;

pub fn stegano_image_benchmark(c: &mut Criterion) {
    let plain_image = image::open("../resources/plain/carrier-image.png")
        .expect("Input image is not readable.")
        .to_rgba();
    let (width, height) = plain_image.dimensions();
    let secret_message = b"Hello World!";

    c.bench_function("SteganoCore Image Encoding to memory", |b| {
        b.iter(|| {
            let mut image_with_secret = image::RgbaImage::new(width, height);
            LSBCodec::encoder(&plain_image, &mut image_with_secret)
                .write_all(&secret_message[..])
                .expect("Cannot write to codec");
        })
    });
}

pub fn stegano_audio_benchmark(c: &mut Criterion) {
    use hound::{WavReader, WavWriter};
    use std::path::Path;
    use stegano_core::carriers::audio::LSBCodec;
    use tempdir::TempDir;

    let input: &Path = "../resources/plain/carrier-audio.wav".as_ref();
    let out_dir = TempDir::new("audio-temp").expect("Cannot create temp dir");
    let audio_with_secret = out_dir.path().join("audio-with-secret.wav");

    let secret_message = b"Hello World!";
    c.bench_function("SteganoCore Audio Encoding to file", |b| {
        b.iter(|| {
            let mut reader = WavReader::open(input).expect("Cannot create reader");
            let mut writer = WavWriter::create(audio_with_secret.as_path(), reader.spec())
                .expect("Cannot create writer");
            LSBCodec::encoder(&mut reader, &mut writer)
                .write_all(&secret_message[..])
                .expect("Cannot write to codec");
        })
    });
}

criterion_group!(benches, stegano_image_benchmark, stegano_audio_benchmark);
criterion_main!(benches);
