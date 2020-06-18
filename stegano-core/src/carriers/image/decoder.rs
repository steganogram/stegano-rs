use crate::lsb::UnveilAlgorithm;
use crate::universal_decoder::CarrierItem;
use image::RgbaImage;
use std::io::Read;

/// stegano source for image files, based on `RgbaImage` by `image` crate
///
/// ## Example of usage
/// ```rust
/// use std::path::Path;
/// use std::io::Read;
/// use image::{RgbaImage};
/// use stegano_core::universal_decoder::Decoder;
/// use stegano_core::carriers::image::decoder::{ImagePngSource, PngUnveil};
///
/// // create a `RgbaImage` from an audio file
/// let mut image = image::open("../resources/with_text/hello_world.png")
///     .expect("Cannot open secret image file")
///     .to_rgba();
/// let mut secret = vec![0; 13];
///
/// // create a `Decoder` based on an `ImagePngSource` based on the `RgbaImage`
/// Decoder::new(ImagePngSource::new(&mut image), PngUnveil)
///     .read_exact(&mut secret)
///     .expect("Cannot read 13 bytes from decoder");
///
/// let msg = String::from_utf8(secret).expect("Cannot convert result to string");
/// assert_eq!("\u{1}Hello World!", msg);
/// ```
pub struct ImagePngSource<'i> {
    pub input: &'i mut RgbaImage,
    max_x: u32,
    max_y: u32,
    max_c: u8,
    x: u32,
    y: u32,
    c: u8,
}

impl<'i> ImagePngSource<'i> {
    /// constructor for a given `RgbaImage` that lives somewhere
    pub fn new(input: &'i mut RgbaImage) -> Self {
        let (max_x, max_y) = input.dimensions();
        Self {
            input,
            max_x,
            max_y,
            max_c: 3,
            x: 0,
            y: 0,
            c: 0,
        }
    }
}

/// iterates over the image and returns single color channels of each pixel
impl<'i> Iterator for ImagePngSource<'i> {
    type Item = CarrierItem;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        if self.x == self.max_x {
            return None;
        }
        let pixel = self.input.get_pixel(self.x, self.y);
        let result = Some(CarrierItem::UnsignedByte(pixel.0[self.c as usize]));
        self.c += 1;
        if self.c == self.max_c {
            self.c = 0;
            self.y += 1;
        }
        if self.y == self.max_y {
            self.y = 0;
            self.x += 1;
        }
        result
    }
}

/// wav specific implementation for unveil of data
pub struct PngUnveil;
impl UnveilAlgorithm<CarrierItem> for PngUnveil {
    #[inline(always)]
    fn decode(&self, carrier: CarrierItem) -> bool {
        match carrier {
            CarrierItem::UnsignedByte(b) => (b & 0x1) > 0,
            CarrierItem::SignedTwoByte(b) => (b & 0x1) > 0,
        }
    }
}
