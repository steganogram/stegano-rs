use criterion::{criterion_group, criterion_main, Criterion};
use hound::WavReader;
use stegano_core::media::audio::LsbCodec;

pub fn audio_encoding(c: &mut Criterion) {
    c.bench_function("Audio Encoding to memory", |b| {
        let mut reader =
            WavReader::open("tests/audio/plain/carrier-audio.wav").expect("Cannot create reader");
        let mut samples = reader.samples().map(|s| s.unwrap()).collect::<Vec<i16>>();
        let secret_message = b"Hello World!";

        b.iter(|| {
            LsbCodec::encoder(&mut samples)
                .write_all(&secret_message[..])
                .expect("Cannot write to codec");
        })
    });
}

criterion_group!(benches, audio_encoding);
criterion_main!(benches);
