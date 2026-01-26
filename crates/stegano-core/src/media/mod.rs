pub mod audio;
pub mod codec_options;
pub mod image;
pub mod payload;
mod primitives;
mod types;

use std::path::Path;

pub use codec_options::{CodecOptions, F5CodecOptions, LsbCodecOptions, DEFAULT_JPEG_QUALITY};
pub use primitives::*;
pub use types::*;

pub trait Persist {
    fn save_as(&mut self, _: &Path) -> crate::Result<()>;
}
