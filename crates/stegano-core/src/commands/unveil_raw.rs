use crate::media::audio::wav_iter::AudioWavIter;
use crate::media::image::LsbCodec;
use crate::media::payload::{FabA, FabS, PayloadCodecFactory};
use crate::universal_decoder::{Decoder, OneBitUnveil};
use crate::{CodecOptions, Media, RawMessage, SteganoError};
use std::fs::File;
use std::io::Write;
use std::path::Path;

/// unveil all raw data, no content format interpretation is happening.
/// Just a raw binary dump of the data gathered by the LSB algorithm.
pub fn unveil_raw(
    secret_media: &Path,
    destination_file: &Path,
    password: Option<String>,
) -> Result<(), SteganoError> {
    let media = Media::from_file(secret_media)?;
    // todo: use this factory in the future
    let _fab: Box<dyn PayloadCodecFactory> = if let Some(password) = password {
        Box::new(FabS::new(password))
    } else {
        Box::new(FabA)
    };

    match media {
        Media::Image(image) => {
            let mut decoder = LsbCodec::decoder(&image, &CodecOptions::default());
            let msg = RawMessage::of(&mut decoder);
            let mut destination_file = File::create(destination_file)
                .map_err(|source| SteganoError::WriteError { source })?;

            destination_file
                .write_all(msg.content.as_slice())
                .map_err(|source| SteganoError::WriteError { source })
        }
        Media::Audio(audio) => {
            let mut decoder = Decoder::new(AudioWavIter::new(audio.1.into_iter()), OneBitUnveil);

            let msg = RawMessage::of(&mut decoder);
            let mut destination_file = File::create(destination_file)
                .map_err(|source| SteganoError::WriteError { source })?;

            destination_file
                .write_all(msg.content.as_slice())
                .map_err(|source| SteganoError::WriteError { source })
        }
    }
}
