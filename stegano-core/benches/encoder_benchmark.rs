use criterion::{criterion_group, criterion_main, Criterion};
use stegano_core::media::image::LsbCodec;

pub fn stegano_image_benchmark(c: &mut Criterion) {
    let plain_image = image::open("../resources/plain/carrier-image.png")
        .expect("Input image is not readable.")
        .to_rgba8();
    let (width, height) = plain_image.dimensions();
    let secret_message = b"Hello World!";

    c.bench_function("SteganoCore Image Encoding to memory", |b| {
        b.iter(|| {
            let mut image_with_secret = image::RgbaImage::new(width, height);
            LsbCodec::encoder(&mut image_with_secret)
                .write_all(&secret_message[..])
                .expect("Cannot write to codec");
        })
    });
}

pub fn stegano_audio_benchmark(c: &mut Criterion) {
    use hound::WavReader;
    use stegano_core::media::audio::LsbCodec;

    let mut reader =
        WavReader::open("../resources/plain/carrier-audio.wav").expect("Cannot create reader");
    let mut samples = reader.samples().map(|s| s.unwrap()).collect::<Vec<i16>>();
    let secret_message = b"Hello World!";

    c.bench_function("SteganoCore Audio Encoding to memory", |b| {
        b.iter(|| {
            LsbCodec::encoder(&mut samples)
                .write_all(&secret_message[..])
                .expect("Cannot write to codec");
        })
    });
}

criterion_group!(benches, stegano_image_benchmark, stegano_audio_benchmark);
criterion_main!(benches);
