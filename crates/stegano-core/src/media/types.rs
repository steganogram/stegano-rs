use std::path::Path;

pub use hound::{WavReader, WavSpec, WavWriter};
pub use image::RgbaImage;
use log::error;

use crate::error::SteganoError;
use crate::media::image::CodecOptions;
use crate::result::Result;

use super::Persist;

pub type WavAudio = (WavSpec, Vec<i16>);

/// a media container for steganography
pub enum Media {
    Image(RgbaImage),
    Audio(WavAudio),
}

impl Media {
    pub fn from_file(f: &Path) -> Result<Self> {
        if let Some(ext) = f.extension() {
            let ext = ext.to_str().unwrap().to_lowercase();
            match ext.as_str() {
                "png" | "jpg" | "jpeg" => Ok(Self::Image(
                    image::open(f)
                        .map_err(|_e| SteganoError::InvalidImageMedia)?
                        .to_rgba8(),
                )),
                "wav" => {
                    let mut reader =
                        WavReader::open(f).map_err(|_e| SteganoError::InvalidAudioMedia)?;
                    let spec = reader.spec();
                    let samples: Vec<i16> = reader.samples().map(|s| s.unwrap()).collect();

                    Ok(Self::Audio((spec, samples)))
                }
                _ => Err(SteganoError::UnsupportedMedia),
            }
        } else {
            Err(SteganoError::UnsupportedMedia)
        }
    }

    pub fn hide_data(&mut self, msg_data: Vec<u8>, opts: &CodecOptions) -> Result<&mut Self> {
        match self {
            Media::Image(i) => {
                let (width, height) = i.dimensions();
                let mut encoder = super::image::LsbCodec::encoder(i, opts);

                encoder.write_all(msg_data.as_ref()).map_err(|e| {
                    error!("Error encoding image: {e}, kind {}", e.kind());

                    match e.kind() {
                        std::io::ErrorKind::WriteZero => {
                            let capacity = width * height;
                            // let ratio = width as f64 / height as f64;
                            let estimated_needed_dimensions = msg_data.len() * 8 / 3;
                            let scale = estimated_needed_dimensions as f64 / capacity as f64;
                            let w = scale * width as f64;
                            let h = scale * height as f64;

                            SteganoError::ImageCapacityError(
                                width as _,
                                height as _,
                                w as _,
                                h as _,
                            )
                        }
                        _ => SteganoError::ImageEncodingError,
                    }
                })?
            }
            Media::Audio((_spec, samples)) => {
                let mut encoder = super::audio::LsbCodec::encoder(samples);

                encoder
                    .write_all(msg_data.as_ref())
                    .map_err(|_e| SteganoError::AudioEncodingError)?
            }
        }

        Ok(self)
    }
}

impl Persist for Media {
    fn save_as(&mut self, file: &Path) -> Result<()> {
        match self {
            Media::Image(i) => i.save(file).map_err(|e| {
                error!("Error saving image to file: {file:?}: {e}");
                SteganoError::ImageEncodingError
            }),
            Media::Audio((spec, samples)) => {
                let mut writer =
                    WavWriter::create(file, *spec).map_err(|_| SteganoError::AudioCreationError)?;
                if let Some(error) = samples
                    .iter()
                    .map(|s| {
                        writer
                            .write_sample(*s)
                            .map_err(|_| SteganoError::AudioEncodingError)
                    })
                    .filter_map(Result::err)
                    .next()
                {
                    return Err(error);
                }

                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_load_jpeg_as_media_image() {
        let media =
            Media::from_file(Path::new("tests/images/NoSecrets.jpg")).expect("Should load JPEG");
        match media {
            Media::Image(img) => {
                assert!(img.width() > 0);
                assert!(img.height() > 0);
            }
            _ => panic!("Expected Media::Image for JPEG input"),
        }
    }
}
