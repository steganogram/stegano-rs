use criterion::{criterion_group, criterion_main, Criterion};
use hound::WavReader;
use stegano_core::media::audio::LsbCodec;

pub fn audio_decoding(c: &mut Criterion) {
    c.bench_function("Audio Decoding", |b| {
        let mut reader = WavReader::open("../resources/secrets/audio-with-secrets.wav")
            .expect("Cannot create reader");
        let mut buf = [0; 12];

        b.iter(|| {
            reader.seek(0).expect("Cannot seek to 0");
            LsbCodec::decoder(&mut reader)
                .read_exact(&mut buf)
                .expect("Cannot read 12 bytes from decoder");
        })
    });
}

criterion_group!(benches, audio_decoding);
criterion_main!(benches);
