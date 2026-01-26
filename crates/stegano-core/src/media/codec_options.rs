/// Codec configuration for steganography encoding/decoding
///
/// The codec choice determines both the encoding method AND output format:
/// - `Lsb` → PNG output (LSB in pixel values)
/// - `F5` → JPEG output (F5 in DCT coefficients)
/// - `AudioLsb` → WAV output (LSB in audio samples)
#[derive(Debug)]
pub enum CodecOptions {
    Lsb(LsbCodecOptions),
    F5(F5CodecOptions),
    AudioLsb,
}

impl Default for CodecOptions {
    fn default() -> Self {
        Self::Lsb(LsbCodecOptions::default())
    }
}

/// Options for LSB (Least Significant Bit) image encoding
#[derive(Debug)]
pub struct LsbCodecOptions {
    /// Determines the step width when iterating over the color channels.
    /// For example `2` would move from (R)GBA to RG(B)A.
    /// Depending on if the alpha channel is skipped (`skip_alpha_channel`) it would either
    /// not count alpha at all or it does.
    ///
    /// For example `2` with alpha skipped would move from RG(B)A to R(G)BA on the next pixel because alpha does not count.
    /// Where as when alpha is not skipped it would move from RG(B)A to (R)GBA on the next pixel.
    ///
    /// Note this number influences the capacity directly.
    pub color_channel_step_increment: usize,

    /// If true no alpha channel would be used for encoding,
    /// this reduces then the capacity by one bit per pixel
    pub skip_alpha_channel: bool,

    /// The concealer strategy, decides on where in a color channel things are going to be stored.
    pub concealer: Concealer,

    /// This limits all iterations to skip the last column and row, in fact it reduces width and height of the image by 1
    pub skip_last_row_and_column: bool,
}

impl Default for LsbCodecOptions {
    fn default() -> Self {
        Self {
            color_channel_step_increment: 1,
            skip_alpha_channel: true,
            concealer: Concealer::LeastSignificantBit,
            skip_last_row_and_column: true,
        }
    }
}

impl LsbCodecOptions {
    pub fn get_color_channel_step_increment(&self) -> usize {
        self.color_channel_step_increment
    }

    #[allow(dead_code)]
    pub fn get_skip_alpha_channel(&self) -> bool {
        self.skip_alpha_channel
    }
}

/// Default JPEG quality for F5 encoding (1-100)
pub const DEFAULT_JPEG_QUALITY: u8 = 90;

/// Options for F5 JPEG steganography encoding
#[derive(Debug)]
pub struct F5CodecOptions {
    /// Seed for F5 embedding (derived from password)
    pub seed: Option<Vec<u8>>,
    /// JPEG quality (1-100, default 90)
    pub quality: u8,
}

impl Default for F5CodecOptions {
    fn default() -> Self {
        Self {
            seed: None,
            quality: DEFAULT_JPEG_QUALITY,
        }
    }
}

impl F5CodecOptions {
    #[allow(dead_code)]
    pub fn with_seed(mut self, seed: Option<Vec<u8>>) -> Self {
        self.seed = seed;
        self
    }

    #[allow(dead_code)]
    pub fn with_quality(mut self, quality: u8) -> Self {
        self.quality = quality;
        self
    }
}

/// Concealer strategy for LSB encoding
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
#[allow(dead_code)]
pub enum Concealer {
    LeastSignificantBit,
    LowFrequencies,
}
