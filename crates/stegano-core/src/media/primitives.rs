/// wrap the low level data types that carries information
#[derive(Debug, Eq, PartialEq)]
pub enum MediaPrimitive {
    ImageColorChannel(u8),
    AudioSample(i16),
}

impl From<u8> for MediaPrimitive {
    fn from(value: u8) -> Self {
        MediaPrimitive::ImageColorChannel(value)
    }
}

/// mutable primitive for storing stegano data
#[derive(Debug, Eq, PartialEq)]
pub enum MediaPrimitiveMut<'a> {
    ImageColorChannel(&'a mut u8),
    AudioSample(&'a mut i16),
}
