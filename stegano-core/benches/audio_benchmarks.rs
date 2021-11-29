use criterion::{criterion_group, criterion_main, Criterion};
use hound::WavReader;
use stegano_core::media::audio::LsbCodec;

pub fn stegano_audio_benchmark(c: &mut Criterion) {
    c.bench_function("Audio Encoding to memory", |b| {
        let mut reader =
            WavReader::open("../resources/plain/carrier-audio.wav").expect("Cannot create reader");
        let mut samples = reader.samples().map(|s| s.unwrap()).collect::<Vec<i16>>();
        let secret_message = b"Hello World!";

        b.iter(|| {
            LsbCodec::encoder(&mut samples)
                .write_all(&secret_message[..])
                .expect("Cannot write to codec");
        })
    });

    c.bench_function("Audio Decoding", |b| {
        let mut reader = WavReader::open("../resources/secrets/audio-with-secrets.wav")
            .expect("Cannot create reader");

        b.iter(|| {
            reader.seek(0).expect("Cannot seek to 0");
            let mut buf = vec![0; 12];
            LsbCodec::decoder(&mut reader)
                .read_exact(&mut buf)
                .expect("Cannot read 12 bytes from decoder");
            let msg = String::from_utf8(buf).expect("Cannot convert result to string");
            assert_eq!("Hello World!", msg);
        })
    });
}

criterion_group!(benches, stegano_audio_benchmark);
criterion_main!(benches);
