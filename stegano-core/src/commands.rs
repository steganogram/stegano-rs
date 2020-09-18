use crate::media::audio::wav_iter::AudioWavIter;
use crate::media::image::LSBCodec;
use crate::universal_decoder::{Decoder, OneBitUnveil};
use crate::{Media, Message, RawMessage, SteganoError};
use std::fs::File;
use std::io::Write;
use std::path::Path;

pub fn unveil(secret_media: &Path, destination: &Path) -> Result<(), SteganoError> {
    let media = Media::from_file(secret_media)?;

    let files = match media {
        Media::Image(image) => {
            let mut decoder = LSBCodec::decoder(&image);
            let msg = Message::of(&mut decoder);
            let mut files = msg.files;

            if let Some(text) = msg.text {
                files.push(("secret-message.txt".to_owned(), text.as_bytes().to_vec()));
            }

            files
        }
        Media::Audio(audio) => {
            let mut decoder = Decoder::new(AudioWavIter::new(audio.1.into_iter()), OneBitUnveil);

            let msg = Message::of(&mut decoder);
            let mut files = msg.files;

            if let Some(text) = msg.text {
                files.push(("secret-message.txt".to_owned(), text.as_bytes().to_vec()));
            }

            files
        }
    };

    if files.is_empty() {
        return Err(SteganoError::NoSecretData);
    }

    for (file_name, buf) in files.iter().map(|(file_name, buf)| {
        let file = Path::new(file_name).file_name().unwrap().to_str().unwrap();

        (file, buf)
    }) {
        let target_file = destination.join(file_name);
        let mut target_file =
            File::create(target_file).map_err(|source| SteganoError::WriteError { source })?;

        target_file
            .write_all(buf.as_slice())
            .map_err(|source| SteganoError::WriteError { source })?;
    }

    Ok(())
}

/// unveil all raw data, no content format interpretation is happening.
/// Just a raw binary dump of the data gathered by the LSB algorithm.
pub fn unveil_raw(secret_media: &Path, destination_file: &Path) -> Result<(), SteganoError> {
    let media = Media::from_file(secret_media)?;

    match media {
        Media::Image(image) => {
            let mut decoder = LSBCodec::decoder(&image);
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
