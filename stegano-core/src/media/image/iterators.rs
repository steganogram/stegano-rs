use image::buffer::{PixelsMut, RowsMut};
use image::Pixel;
use std::slice::IterMut;

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

pub(crate) struct ColorIter<'a, P: Pixel + 'a> {
    pixel: TransposeMut<'a, P>,
    colors: IterMut<'a, P::Subpixel>,
}

impl<'a, P: Pixel + 'a> ColorIter<'a, P> {
    pub fn from_transpose(mut t: TransposeMut<'a, P>) -> Self {
        let i = t.next().unwrap().channels_mut().iter_mut();
        Self {
            pixel: t,
            colors: i,
        }
    }
}

impl<'a, P: Pixel + 'a> Iterator for ColorIter<'a, P> {
    type Item = &'a mut P::Subpixel;

    fn next(&mut self) -> Option<Self::Item> {
        self.colors.next().or_else(|| {
            self.colors = self.pixel.next().unwrap().channels_mut().iter_mut();
            self.colors.next()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;

    const HELLO_WORLD_PNG: &str = "../resources/with_text/hello_world.png";

    #[test]
    fn transpose_mut() {
        let mut img = image::open(HELLO_WORLD_PNG)
            .expect("Input image is not readable.")
            .to_rgba();
        let mut img_ref = image::open(HELLO_WORLD_PNG)
            .expect("Input image is not readable.")
            .to_rgba();
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
            .to_rgba();
        let mut img_ref = image::open(HELLO_WORLD_PNG)
            .expect("Input image is not readable.")
            .to_rgba();
        let (width, height) = img.dimensions();
        let mut c_iter =
            ColorIter::from_transpose(TransposeMut::from_rows_mut(img.rows_mut(), height));

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
}
