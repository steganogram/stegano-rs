use image::buffer::{Pixels, PixelsMut, Rows, RowsMut};
use image::Pixel;
use std::slice::{Iter, IterMut};

/// Allows transposed mutable access to pixel, like column based
pub(crate) struct TransposeMut<'a, P: Pixel + 'a> {
    i: usize,
    height: u32,
    rows_mut: RowsMut<'a, P>,
    rows: Vec<PixelsMut<'a, P>>,
}

impl<'a, P: Pixel + 'a> TransposeMut<'a, P> {
    /// utilises RowsMut to give Column based mut access to pixel
    pub fn from_rows_mut(rows_mut: RowsMut<'a, P>, height: u32) -> Self {
        Self {
            i: 0,
            height,
            rows_mut,
            rows: Vec::with_capacity(height as usize),
        }
    }
}

impl<'a, P: Pixel + 'a> Iterator for TransposeMut<'a, P> {
    type Item = &'a mut P;

    fn next(&mut self) -> Option<Self::Item> {
        let row_idx = ((self.i as u32) % self.height) as usize;
        self.i += 1;
        match self.rows.get_mut(row_idx) {
            None => match self.rows_mut.next() {
                Some(mut row) => {
                    let p = row.next();
                    self.rows.push(row);
                    p
                }
                _ => None,
            },
            Some(row) => row.next(),
        }
    }
}

pub(crate) struct Transpose<'a, P: Pixel + 'a> {
    i: usize,
    height: u32,
    rows: Rows<'a, P>,
    rows_buffer: Vec<Pixels<'a, P>>,
}

impl<'a, P: Pixel + 'a> Transpose<'a, P> {
    /// utilizes Rows to give column based readonly access to pixel
    pub fn from_rows(rows: Rows<'a, P>, height: u32) -> Self {
        Self {
            i: 0,
            height,
            rows,
            rows_buffer: Vec::with_capacity(height as usize),
        }
    }
}

impl<'a, P: Pixel + 'a> Iterator for Transpose<'a, P> {
    type Item = &'a P;

    fn next(&mut self) -> Option<Self::Item> {
        let row_idx = ((self.i as u32) % self.height) as usize;
        self.i += 1;
        match self.rows_buffer.get_mut(row_idx) {
            None => match self.rows.next() {
                Some(mut row) => {
                    let p = row.next();
                    self.rows_buffer.push(row);
                    p
                }
                _ => None,
            },
            Some(row) => row.next(),
        }
    }
}

pub(crate) struct ColorIterMut<'a, P: Pixel + 'a> {
    pixel: TransposeMut<'a, P>,
    colors: IterMut<'a, P::Subpixel>,
}

impl<'a, P: Pixel + 'a> ColorIterMut<'a, P> {
    pub fn from_transpose(mut t: TransposeMut<'a, P>) -> Self {
        let i = t.next().unwrap().channels_mut().iter_mut();
        Self {
            pixel: t,
            colors: i,
        }
    }
}

impl<'a, P: Pixel + 'a> Iterator for ColorIterMut<'a, P> {
    type Item = &'a mut P::Subpixel;

    fn next(&mut self) -> Option<Self::Item> {
        self.colors.next().or_else(|| {
            if let Some(iter) = self.pixel.next() {
                self.colors = iter.channels_mut().iter_mut();
            }
            self.colors.next()
        })
    }
}

pub(crate) struct ColorIter<'a, P: Pixel + 'a> {
    pixel: Transpose<'a, P>,
    colors: Iter<'a, P::Subpixel>,
}

impl<'a, P: Pixel + 'a> ColorIter<'a, P> {
    pub fn from_transpose(mut t: Transpose<'a, P>) -> Self {
        let i = t.next().unwrap().channels().iter();
        Self {
            pixel: t,
            colors: i,
        }
    }
}

impl<'a, P: Pixel + 'a> Iterator for ColorIter<'a, P> {
    type Item = &'a P::Subpixel;

    fn next(&mut self) -> Option<Self::Item> {
        self.colors.next().or_else(|| {
            if let Some(iter) = self.pixel.next() {
                self.colors = iter.channels().iter();
            }
            self.colors.next()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{ImageBuffer, Rgba, RgbaImage};

    const HELLO_WORLD_PNG: &str = "../resources/with_text/hello_world.png";

    fn prepare_small_image() -> RgbaImage {
        ImageBuffer::from_fn(5, 5, |x, y| {
            let i = (4 * x + 20 * y) as u8;
            image::Rgba([i, i + 1, i + 2, i + 3])
        })
    }

    #[test]
    fn transpose_mut() {
        let mut img = image::open(HELLO_WORLD_PNG)
            .expect("Input image is not readable.")
            .to_rgba8();
        let mut img_ref = image::open(HELLO_WORLD_PNG)
            .expect("Input image is not readable.")
            .to_rgba8();
        let (width, height) = img.dimensions();
        let mut t = TransposeMut::from_rows_mut(img.rows_mut(), height);

        for x in 0..width {
            for y in 0..height {
                let pi = t.next().unwrap();
                let p1 = img_ref.get_pixel_mut(x, y);
                assert_eq!(p1, pi, "Pixel ({}, {}) does not match", x, y);
                *pi = Rgba([33, 33, 33, 33]);
            }
        }
        assert_eq!(t.next(), None, "Iterator should be exhausted");
        let t = TransposeMut::from_rows_mut(img.rows_mut(), height);
        for pi in t {
            assert_eq!(
                pi,
                &Rgba([33, 33, 33, 33]),
                "Pixel should have been mutated earlier"
            );
        }
    }

    #[test]
    fn iter_color() {
        let mut img = image::open(HELLO_WORLD_PNG)
            .expect("Input image is not readable.")
            .to_rgba8();
        let mut img_ref = image::open(HELLO_WORLD_PNG)
            .expect("Input image is not readable.")
            .to_rgba8();
        let (width, height) = img.dimensions();
        let mut c_iter =
            ColorIterMut::from_transpose(TransposeMut::from_rows_mut(img.rows_mut(), height));

        for x in 0..width {
            for y in 0..height {
                for c in 0..4 {
                    let actual_color = c_iter.next().unwrap();
                    let p1 = img_ref.get_pixel_mut(x, y);
                    let expected_color = p1.0.get_mut(c).unwrap();

                    assert_eq!(
                        expected_color, actual_color,
                        "Pixel ({}, {}) colors does not match",
                        x, y
                    );
                    // *pi = Rgba([33, 33, 33, 33]);
                }
            }
        }
        // assert_eq!(t.next(), None, "Iterator should be exhausted");
        // let t = TransposeMut::from_rows_mut(img.rows_mut(), height);
        // for pi in t {
        //     assert_eq!(
        //         pi,
        //         &Rgba([33, 33, 33, 33]),
        //         "Pixel should have been mutated earlier"
        //     );
        // }
    }

    #[cfg(test)]
    mod color_iter {
        use crate::media::image::iterators::tests::prepare_small_image;
        use crate::media::image::iterators::*;
        use image::Rgba;

        #[test]
        fn should_transpose_read() {
            let img = prepare_small_image();
            let mut iter = Transpose::from_rows(img.rows(), img.height());

            assert_eq!(iter.next(), Some(&Rgba([0_u8, 1, 2, 3])));
            assert_eq!(iter.next(), Some(&Rgba([20_u8, 21, 22, 23])));
            assert_eq!(iter.next(), Some(&Rgba([40_u8, 41, 42, 43])));
        }

        #[test]
        fn should_read_color() {
            let img = prepare_small_image();
            let iter = Transpose::from_rows(img.rows(), img.height());
            let mut color_iter = ColorIter::from_transpose(iter);

            assert_eq!(color_iter.next(), Some(&0_u8));
            assert_eq!(color_iter.next(), Some(&1_u8));
            assert_eq!(color_iter.next(), Some(&2_u8));
            assert_eq!(color_iter.next(), Some(&3_u8));
            assert_eq!(color_iter.next(), Some(&20_u8));
            assert_eq!(color_iter.next(), Some(&21_u8));
        }
    }
}
