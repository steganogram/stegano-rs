/// generic stegano decoder
pub(crate) struct Decoder<'i, I, P, A> {
    pub(crate) input: &'i mut I,
    pub(crate) algorithm: A,
    pub(crate) position: P,
}

/// generic stegano decoder constructor method
impl<'i, I, P, A> Decoder<'i, I, P, A> {
    pub fn new(input: &'i mut I, algorithm: A, position: P) -> Self {
        Decoder {
            input,
            algorithm,
            position,
        }
    }
}
