use std::io::Write;
use std::path::Path;

pub use hound::{WavReader, WavSpec, WavWriter};
pub use image::RgbaImage;
use log::error;

use crate::error::SteganoError;
use crate::media::codec_options::{CodecOptions, LsbCodecOptions};
use crate::result::Result;

use super::Persist;

pub type WavAudio = (WavSpec, Vec<i16>);

/// A media container for steganography
pub enum Media {
    /// PNG image - stores decoded pixels
    Image(RgbaImage),
    /// JPEG image - stores both decoded pixels and original source bytes for F5 transcoding
    ImageJpeg { pixels: RgbaImage, source: Vec<u8> },
    /// WAV audio
    Audio(WavAudio),
}

/// Result of encoding data into media
pub enum EncodedMedia {
    /// Modified pixels ready to encode as PNG
    Png(RgbaImage),
    /// Complete JPEG bytes ready to write
    Jpeg(Vec<u8>),
    /// Modified audio samples ready to encode as WAV
    Wav(WavSpec, Vec<i16>),
}

impl Media {
    pub fn from_file(f: &Path) -> Result<Self> {
        if let Some(ext) = f.extension() {
            let ext = ext.to_str().unwrap().to_lowercase();
            match ext.as_str() {
                "png" => Ok(Self::Image(
                    image::open(f)
                        .map_err(|_e| SteganoError::InvalidImageMedia)?
                        .to_rgba8(),
                )),
                "jpg" | "jpeg" => {
                    let source =
                        std::fs::read(f).map_err(|source| SteganoError::ReadError { source })?;
                    let pixels = image::open(f)
                        .map_err(|_e| SteganoError::InvalidImageMedia)?
                        .to_rgba8();
                    Ok(Self::ImageJpeg { pixels, source })
                }
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

    /// Hide data in the media using the specified codec options
    ///
    /// Returns an `EncodedMedia` ready to be saved
    pub fn hide_data(&self, msg_data: Vec<u8>, opts: &CodecOptions) -> Result<EncodedMedia> {
        match (self, opts) {
            // LSB encoding for images → PNG output
            (Media::Image(img), CodecOptions::Lsb(lsb_opts)) => {
                let mut img = img.clone();
                Self::encode_lsb(&mut img, &msg_data, lsb_opts)?;
                Ok(EncodedMedia::Png(img))
            }
            (Media::ImageJpeg { pixels, .. }, CodecOptions::Lsb(lsb_opts)) => {
                let mut img = pixels.clone();
                Self::encode_lsb(&mut img, &msg_data, lsb_opts)?;
                Ok(EncodedMedia::Png(img))
            }

            // F5 encoding → JPEG output
            (Media::Image(img), CodecOptions::F5(f5_opts)) => {
                // PNG → JPEG: encode from pixels
                let rgba = img.as_flat_samples().samples;
                let (width, height) = img.dimensions();
                let jpeg_data = stegano_f5::embed_in_jpeg_from_image(
                    rgba,
                    width as u16,
                    height as u16,
                    f5_opts.quality,
                    stegano_f5_jpeg_encoder::ColorType::Rgba,
                    &msg_data,
                    f5_opts.seed.as_deref(),
                )
                .map_err(|e| SteganoError::JpegError {
                    reason: e.to_string(),
                })?;
                Ok(EncodedMedia::Jpeg(jpeg_data))
            }
            (Media::ImageJpeg { source, .. }, CodecOptions::F5(f5_opts)) => {
                // JPEG → JPEG: transcode preserving characteristics
                let jpeg_data =
                    stegano_f5::embed_in_jpeg(source, &msg_data, f5_opts.seed.as_deref()).map_err(
                        |e| SteganoError::JpegError {
                            reason: e.to_string(),
                        },
                    )?;
                Ok(EncodedMedia::Jpeg(jpeg_data))
            }

            // Audio LSB encoding → WAV output
            (Media::Audio((spec, samples)), CodecOptions::AudioLsb) => {
                let mut samples = samples.clone();
                {
                    let mut encoder = super::audio::LsbCodec::encoder(&mut samples);
                    encoder
                        .write_all(&msg_data)
                        .map_err(|_e| SteganoError::AudioEncodingError)?;
                }
                Ok(EncodedMedia::Wav(*spec, samples))
            }

            // Invalid combinations
            (Media::Audio(_), CodecOptions::Lsb(_) | CodecOptions::F5(_)) => {
                Err(SteganoError::UnsupportedMedia)
            }
            (Media::Image(_) | Media::ImageJpeg { .. }, CodecOptions::AudioLsb) => {
                Err(SteganoError::UnsupportedMedia)
            }
        }
    }

    fn encode_lsb(img: &mut RgbaImage, msg_data: &[u8], opts: &LsbCodecOptions) -> Result<()> {
        let (width, height) = img.dimensions();
        let mut encoder = super::image::LsbCodec::encoder(img, opts);

        encoder.write_all(msg_data).map_err(|e| {
            error!("Error encoding image: {e}, kind {}", e.kind());

            match e.kind() {
                std::io::ErrorKind::WriteZero => {
                    let capacity = width * height;
                    let estimated_needed_dimensions = msg_data.len() * 8 / 3;
                    let scale = estimated_needed_dimensions as f64 / capacity as f64;
                    let w = scale * width as f64;
                    let h = scale * height as f64;

                    SteganoError::ImageCapacityError(width as _, height as _, w as _, h as _)
                }
                _ => SteganoError::ImageEncodingError,
            }
        })?;
        Ok(())
    }
}

impl Persist for EncodedMedia {
    fn save_as(&mut self, file: &Path) -> Result<()> {
        match self {
            EncodedMedia::Png(img) => {
                // Validate extension
                if let Some(ext) = file.extension().and_then(|e| e.to_str()) {
                    if ext.to_lowercase() != "png" {
                        return Err(SteganoError::FormatMismatch {
                            expected: "png".to_string(),
                            actual: ext.to_string(),
                        });
                    }
                }
                img.save(file).map_err(|e| {
                    error!("Error saving PNG to file: {file:?}: {e}");
                    SteganoError::ImageEncodingError
                })
            }
            EncodedMedia::Jpeg(data) => {
                // Validate extension
                if let Some(ext) = file.extension().and_then(|e| e.to_str()) {
                    let ext_lower = ext.to_lowercase();
                    if ext_lower != "jpg" && ext_lower != "jpeg" {
                        return Err(SteganoError::FormatMismatch {
                            expected: "jpg/jpeg".to_string(),
                            actual: ext.to_string(),
                        });
                    }
                }
                std::fs::write(file, data).map_err(|source| SteganoError::WriteError { source })
            }
            EncodedMedia::Wav(spec, samples) => {
                let mut writer =
                    WavWriter::create(file, *spec).map_err(|_| SteganoError::AudioCreationError)?;
                for sample in samples.iter() {
                    writer
                        .write_sample(*sample)
                        .map_err(|_| SteganoError::AudioEncodingError)?;
                }
                Ok(())
            }
        }
    }
}

// Keep the old Persist impl for backward compatibility during transition
impl Persist for Media {
    fn save_as(&mut self, file: &Path) -> Result<()> {
        match self {
            Media::Image(i) => i.save(file).map_err(|e| {
                error!("Error saving image to file: {file:?}: {e}");
                SteganoError::ImageEncodingError
            }),
            Media::ImageJpeg { pixels, .. } => pixels.save(file).map_err(|e| {
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
    fn should_load_jpeg_as_media_image_jpeg() {
        let media =
            Media::from_file(Path::new("tests/images/NoSecrets.jpg")).expect("Should load JPEG");
        match media {
            Media::ImageJpeg { pixels, source } => {
                assert!(pixels.width() > 0);
                assert!(pixels.height() > 0);
                assert!(!source.is_empty());
                // Verify it's a valid JPEG
                assert_eq!(&source[0..2], &[0xFF, 0xD8]);
            }
            _ => panic!("Expected Media::ImageJpeg for JPEG input"),
        }
    }

    #[test]
    fn should_load_png_as_media_image() {
        let media = Media::from_file(Path::new("tests/images/plain/carrier-image.png"))
            .expect("Should load PNG");
        match media {
            Media::Image(img) => {
                assert!(img.width() > 0);
                assert!(img.height() > 0);
            }
            _ => panic!("Expected Media::Image for PNG input"),
        }
    }
}
