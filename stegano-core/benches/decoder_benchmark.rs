use criterion::{criterion_group, criterion_main, Criterion};
use std::io::Read;
use stegano_core::carriers::image::decoder::{ImagePngSource, PngUnveil};
use stegano_core::universal_decoder::Decoder;
use stegano_core::LSBCodec;

pub fn stegano_image_benchmark(c: &mut Criterion) {
    let mut img = image::open("../resources/with_text/hello_world.png")
        .expect("Input image is not readable.")
        .to_rgba();

    c.bench_function("SteganoCore Image Decoding", |b| {
        b.iter(|| {
            let mut buf = vec![0; 13];
            Decoder::new(ImagePngSource::new(&mut img), PngUnveil)
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

    let input: &Path = "../resources/plain/carrier-audio.wav".as_ref();
    let out_dir = TempDir::new("audio-temp").expect("Cannot create temp dir");
    let audio_with_secret = out_dir.path().join("audio-with-secret.wav");

    let secret_message = "Hello World!".as_bytes();
    c.bench_function("SteganoCore Audio Encoding", |b| {
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
