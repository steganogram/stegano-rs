pub mod audio;
pub mod image;
pub mod payload;
mod primitives;
mod types;

use std::path::Path;

pub use primitives::*;
pub use types::*;

pub trait Persist {
    fn save_as(&mut self, _: &Path) -> crate::Result<()>;
}
