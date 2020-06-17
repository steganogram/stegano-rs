/// generic stegano encoder
pub(crate) struct Encoder<'i, I, O, A, P> {
    pub(crate) input: &'i mut I,
    pub(crate) output: &'i mut O,
    pub(crate) algorithm: A,
    pub(crate) position: P,
}

/// generic stegano encoder constructor method
impl<'i, I, O, A, P> Encoder<'i, I, O, A, P> {
    pub fn new(input: &'i mut I, output: &'i mut O, algorithm: A, position: P) -> Self {
        Encoder {
            input,
            output,
            algorithm,
            position,
        }
    }
}
