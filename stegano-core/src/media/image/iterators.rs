use image::buffer::{Pixels, PixelsMut, Rows, RowsMut};
use image::Pixel;
use std::iter::Take;
use std::slice::{Iter, IterMut};

/// Allows transposed mutable access to pixel, like column based
pub(crate) struct TransposeMut<'a, P: Pixel + 'a> {
    i: usize,
    i_max: usize,
    use_max_columns: u32,
    rows_mut: Take<RowsMut<'a, P>>,
    rows: Vec<PixelsMut<'a, P>>,
}

impl<'a, P: Pixel + 'a> TransposeMut<'a, P> {
    /// utilises RowsMut to give Column based mut access to pixel
    pub fn from_rows_mut(
        rows_mut: RowsMut<'a, P>,
        height: u32,
        skip_last_row_and_column: bool,
    ) -> Self {
        let width = rows_mut.len();
        let (rows_mut, height) = if skip_last_row_and_column {
            (rows_mut.take(width - 1), height - 1)
        } else {
            (rows_mut.take(width), height)
        };

        Self {
            i: 0,
            i_max: height as usize * rows_mut.len(),
            use_max_columns: height,
            rows_mut,
            rows: Vec::with_capacity(height as usize),
        }
    }
}

impl<'a, P: Pixel + 'a> Iterator for TransposeMut<'a, P> {
    type Item = &'a mut P;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i == self.i_max {
            return None;
        }
        let row_idx = ((self.i as u32) % self.use_max_columns) as usize;
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
    i_max: usize,
    use_max_columns: u32,
    rows: Take<Rows<'a, P>>,
    rows_buffer: Vec<Pixels<'a, P>>,
}

impl<'a, P: Pixel + 'a> Transpose<'a, P> {
    /// utilizes Rows to give column based readonly access to pixel
    pub fn from_rows(rows: Rows<'a, P>, height: u32, skip_last_row_and_column: bool) -> Self {
        let width = rows.len();
        let (rows, height) = if skip_last_row_and_column {
            (rows.take(width - 1), height - 1)
        } else {
            (rows.take(width), height)
        };

        Self {
            i: 0,
            use_max_columns: height,
            i_max: height as usize * rows.len(),
            rows,
            rows_buffer: Vec::with_capacity(height as usize),
        }
    }
}

impl<'a, P: Pixel + 'a> Iterator for Transpose<'a, P> {
    type Item = &'a P;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i == self.i_max {
            return None;
        }
        let row_idx = ((self.i as u32) % self.use_max_columns) as usize;
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
    colors: Take<IterMut<'a, P::Subpixel>>,
    take: u8,
}

impl<'a, P: Pixel + 'a> ColorIterMut<'a, P> {
    pub fn from_transpose(mut t: TransposeMut<'a, P>, skip_alpha: bool) -> Self {
        let take: u8 = if skip_alpha { 3 } else { 4 };
        let i = t
            .next()
            .unwrap()
            .channels_mut()
            .iter_mut()
            .take(take as usize);
        Self {
            pixel: t,
            colors: i,
            take,
        }
    }
}

impl<'a, P: Pixel + 'a> Iterator for ColorIterMut<'a, P> {
    type Item = &'a mut P::Subpixel;

    fn next(&mut self) -> Option<Self::Item> {
        self.colors.next().or_else(|| {
            if let Some(iter) = self.pixel.next() {
                self.colors = iter.channels_mut().iter_mut().take(self.take as usize);
            }
            self.colors.next()
        })
    }
}

pub(crate) struct ColorIter<'a, P: Pixel + 'a> {
    pixel: Transpose<'a, P>,
    colors: Take<Iter<'a, P::Subpixel>>,
    take: u8,
}

impl<'a, P: Pixel + 'a> ColorIter<'a, P> {
    pub fn from_transpose(mut t: Transpose<'a, P>, skip_alpha: bool) -> Self {
        let take: u8 = if skip_alpha { 3 } else { 4 };
        let i = t.next().unwrap().channels().iter().take(take as usize);
        Self {
            pixel: t,
            colors: i,
            take,
        }
    }
}

impl<'a, P: Pixel + 'a> Iterator for ColorIter<'a, P> {
    type Item = &'a P::Subpixel;

    fn next(&mut self) -> Option<Self::Item> {
        self.colors.next().or_else(|| {
            if let Some(iter) = self.pixel.next() {
                self.colors = iter.channels().iter().take(self.take as usize);
            }
            self.colors.next()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::HELLO_WORLD_PNG;
    use image::Rgba;

    #[test]
    fn transpose_mut() {
        let mut img = image::open(HELLO_WORLD_PNG)
            .expect("Input image is not readable.")
            .to_rgba8();
        let mut img_ref = image::open(HELLO_WORLD_PNG)
            .expect("Input image is not readable.")
            .to_rgba8();
        let (width, height) = img.dimensions();
        let mut t = TransposeMut::from_rows_mut(img.rows_mut(), height, true);

        for x in 0..width {
            for y in 0..height {
                let pi = t.next().unwrap();
                let p1 = img_ref.get_pixel_mut(x, y);
                assert_eq!(p1, pi, "Pixel ({}, {}) does not match", x, y);
                *pi = Rgba([33, 33, 33, 33]);
            }
        }
        assert_eq!(t.next(), None, "Iterator should be exhausted");
        let t = TransposeMut::from_rows_mut(img.rows_mut(), height, true);
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
        let mut c_iter = ColorIterMut::from_transpose(
            TransposeMut::from_rows_mut(img.rows_mut(), height, true),
            true,
        );

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
        use crate::media::image::iterators::*;
        use crate::test_utils::{
            prepare_5x5_image, prepare_5x5_linear_growing_colors_except_last_row_column,
        };
        use image::Rgba;

        #[test]
        fn should_transpose_read() {
            let img = prepare_5x5_image();
            let mut iter = Transpose::from_rows(img.rows(), img.height(), false);

            // first column
            assert_eq!(iter.next(), Some(&Rgba([0_u8, 1, 2, 3])));
            assert_eq!(iter.next(), Some(&Rgba([20_u8, 21, 22, 23])));
            assert_eq!(iter.next(), Some(&Rgba([40_u8, 41, 42, 43])));
            assert_eq!(iter.next(), Some(&Rgba([60_u8, 61, 62, 63])));
            assert_eq!(iter.next(), Some(&Rgba([80_u8, 81, 82, 83])));

            // second column
            assert_eq!(iter.next(), Some(&Rgba([4_u8, 5, 6, 7])));
            assert_eq!(iter.next(), Some(&Rgba([24_u8, 25, 26, 27])));
            assert_eq!(iter.next(), Some(&Rgba([44_u8, 45, 46, 47])));
        }

        #[test]
        fn should_read_color() {
            let img = prepare_5x5_image();
            let iter = Transpose::from_rows(img.rows(), img.height(), false);
            let mut color_iter = ColorIter::from_transpose(iter, true);

            // first row.. alpha skipped
            assert_eq!(color_iter.next(), Some(&0_u8));
            assert_eq!(color_iter.next(), Some(&1_u8));
            assert_eq!(color_iter.next(), Some(&2_u8));
            // 2nd row
            assert_eq!(color_iter.next(), Some(&20_u8));
            assert_eq!(color_iter.next(), Some(&21_u8));
            assert_eq!(color_iter.next(), Some(&22_u8));
            // 3rd row
            assert_eq!(color_iter.next(), Some(&40_u8));
            assert_eq!(color_iter.next(), Some(&41_u8));
            assert_eq!(color_iter.next(), Some(&42_u8));
            // 4rd row
            assert_eq!(color_iter.next(), Some(&60_u8));
            assert_eq!(color_iter.next(), Some(&61_u8));
            assert_eq!(color_iter.next(), Some(&62_u8));
            // 5rd row
            assert_eq!(color_iter.next(), Some(&80_u8));
            assert_eq!(color_iter.next(), Some(&81_u8));
            assert_eq!(color_iter.next(), Some(&82_u8));
        }

        /// this test ensures the transponation works
        /// and the end is working and the lost rows and columns are always skipped
        #[test]
        fn should_ensure_transpose_works_really() {
            let img = prepare_5x5_linear_growing_colors_except_last_row_column();
            let iter = Transpose::from_rows(img.rows(), img.height(), true);
            let color_iter = ColorIter::from_transpose(iter, true);

            for (i, c) in color_iter.enumerate() {
                let i: u8 = i as u8;
                assert_eq!(c, &i, "the ({i}+1)-th color was wrong");
            }
        }

        #[test]
        fn should_ensure_transpose_mut_works_really() {
            let mut img = prepare_5x5_linear_growing_colors_except_last_row_column();
            let height = img.height();
            let mut iter = TransposeMut::from_rows_mut(img.rows_mut(), height, true);
            let color_iter = ColorIterMut::from_transpose(iter, true);

            for (i, c) in color_iter.enumerate() {
                let i: u8 = i as u8;
                assert_eq!(c, &i, "the ({i}+1)-th color was wrong");
            }
        }
    }
}
