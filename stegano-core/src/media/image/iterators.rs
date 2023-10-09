use image::buffer::{Pixels, PixelsMut, Rows, RowsMut};
use image::Pixel;
use std::iter::Take;
use std::ops::Sub;
use std::slice::{Iter, IterMut};

/// Allows transposed mutable access to pixel, like column based
pub(crate) struct TransposeMut<'a, P: Pixel + 'a> {
    i: usize,
    i_max: usize,
    use_max_rows: u32,
    rows_mut: Take<RowsMut<'a, P>>,
    rows_buffer: Vec<PixelsMut<'a, P>>,
}

impl<'a, P: Pixel + 'a> TransposeMut<'a, P> {
    /// utilises RowsMut to give Column based mut access to pixel
    pub fn from_rows_mut(
        rows_mut: RowsMut<'a, P>,
        width: u32,
        skip_last_row_and_column: bool,
    ) -> Self {
        let (height, width) = if skip_last_row_and_column {
            (rows_mut.len() - 1, width.sub(1) as usize)
        } else {
            (rows_mut.len(), width as usize)
        };

        Self {
            i: 0,
            i_max: height * width,
            use_max_rows: height as _,
            rows_mut: rows_mut.take(height),
            rows_buffer: Vec::with_capacity(height),
        }
    }
}

impl<'a, P: Pixel + 'a> Iterator for TransposeMut<'a, P> {
    type Item = &'a mut P;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i == self.i_max {
            return None;
        }
        let row_idx = ((self.i as u32) % self.use_max_rows) as usize;
        self.i += 1;
        match self.rows_buffer.get_mut(row_idx) {
            None => match self.rows_mut.next() {
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

pub(crate) struct Transpose<'a, P: Pixel + 'a> {
    i: usize,
    i_max: usize,
    use_max_rows: u32,
    rows: Take<Rows<'a, P>>,
    rows_buffer: Vec<Pixels<'a, P>>,
}

impl<'a, P: Pixel + 'a> Transpose<'a, P> {
    /// utilizes Rows to give column based readonly access to pixel
    pub fn from_rows(rows: Rows<'a, P>, width: u32, skip_last_row_and_column: bool) -> Self {
        let (height, width) = if skip_last_row_and_column {
            (rows.len() - 1, width.sub(1) as usize)
        } else {
            (rows.len(), width as usize)
        };

        Self {
            i: 0,
            i_max: height * width,
            use_max_rows: height as _,
            rows: rows.take(height),
            rows_buffer: Vec::with_capacity(height),
        }
    }
}

impl<'a, P: Pixel + 'a> Iterator for Transpose<'a, P> {
    type Item = &'a P;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i == self.i_max {
            return None;
        }
        let row_idx = ((self.i as u32) % self.use_max_rows) as usize;
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

    use crate::test_utils::{
        prepare_4x6_linear_growing_colors_except_last_row_column_skipped_alpha,
        prepare_4x6_linear_growing_colors_regular_skipped_alpha,
    };

    #[test]
    fn should_ensure_transpose_works_for_regular_image() {
        let mut img = prepare_4x6_linear_growing_colors_regular_skipped_alpha();
        let width = img.width();
        assert_eq!(width, 4);
        assert_eq!(img.height(), 6);
        assert_eq!(img.rows().len(), img.height() as _);
        let iter = Transpose::from_rows(img.rows(), width, false);
        let color_iter = ColorIter::from_transpose(iter, true);
        for (i, c) in color_iter.enumerate() {
            let i: u8 = i as u8;
            assert_eq!(c, &i, "the ({i}+1)-th color was wrong");
        }

        // now the mut iterator
        let iter = TransposeMut::from_rows_mut(img.rows_mut(), width, false);
        let color_iter = ColorIterMut::from_transpose(iter, true);
        for (i, c) in color_iter.enumerate() {
            let i: u8 = i as u8;
            assert_eq!(c, &i, "the ({i}+1)-th color was wrong");
        }
    }

    #[test]
    fn should_ensure_transpose_works_for_images_without_last_row_and_column() {
        let mut img = prepare_4x6_linear_growing_colors_except_last_row_column_skipped_alpha();
        let width = img.width();
        let iter = Transpose::from_rows(img.rows(), width, true);
        let color_iter = ColorIter::from_transpose(iter, true);
        for (i, c) in color_iter.enumerate() {
            let i: u8 = i as u8;
            assert_eq!(c, &i, "the ({i}+1)-th color was wrong");
        }

        // now the mut iterator
        let iter = TransposeMut::from_rows_mut(img.rows_mut(), width, true);
        let color_iter = ColorIterMut::from_transpose(iter, true);
        for (i, c) in color_iter.enumerate() {
            let i: u8 = i as u8;
            assert_eq!(c, &i, "the ({i}+1)-th color was wrong");
        }
    }

    #[test]
    fn ensure_transpose_pixel_iterator_transposes_correctly() {
        let img = prepare_4x6_linear_growing_colors_except_last_row_column_skipped_alpha();
        let (width, height) = img.dimensions();
        let mut iter = Transpose::from_rows(img.rows(), width, true);

        for x in 0..(width - 1) {
            for y in 0..(height - 1) {
                let expected_pixel = img.get_pixel(x, y);
                let given_pixel = iter
                    .next()
                    .unwrap_or_else(|| panic!("Pixel at ({x}, {y}) was not even existing!"));

                assert_eq!(
                    given_pixel, expected_pixel,
                    "Pixel at ({x}, {y}) does not match"
                );
            }
        }
        // ensure iterator is exhausted
        assert!(iter.next().is_none());
    }

    #[test]
    fn ensure_color_iterator_transposes_correctly_with_alpha_channel() {
        let img = prepare_4x6_linear_growing_colors_except_last_row_column_skipped_alpha();
        let (width, height) = img.dimensions();
        let iter = Transpose::from_rows(img.rows(), width, true);
        let mut color_iter = ColorIter::from_transpose(iter, false);

        for x in 0..(width - 1) {
            for y in 0..(height - 1) {
                let expected_pixel = img.get_pixel(x, y);
                for color_idx in 0..4 {
                    let expected_color = expected_pixel.0.get(color_idx).unwrap();
                    let given_color = color_iter
                        .next()
                        .unwrap_or_else(|| panic!("Color at ({x}, {y}) was not even existing!"));

                    assert_eq!(
                        given_color, expected_color,
                        "Color at ({x}, {y}) does not match"
                    );
                }
            }
        }
        // ensure iterator is exhausted
        assert!(color_iter.next().is_none());

        let iter = Transpose::from_rows(img.rows(), width, true);
        let color_iter = ColorIter::from_transpose(iter, false);

        let last_pixel = img.get_pixel(width - 2, height - 2);
        let given_last_color = color_iter.last();
        assert_eq!(last_pixel.0.last(), given_last_color);
    }

    #[test]
    fn ensure_color_iterator_transposes_correctly_without_alpha_channel() {
        let img = prepare_4x6_linear_growing_colors_except_last_row_column_skipped_alpha();
        let (width, height) = img.dimensions();
        let iter = Transpose::from_rows(img.rows(), width, true);
        let mut color_iter = ColorIter::from_transpose(iter, true);

        for x in 0..(width - 1) {
            for y in 0..(height - 1) {
                let expected_pixel = img.get_pixel(x, y);
                for color_idx in 0..3 {
                    let expected_color = expected_pixel.0.get(color_idx).unwrap();
                    let given_color = color_iter
                        .next()
                        .unwrap_or_else(|| panic!("Color at ({x}, {y}) was not even existing!"));

                    assert_eq!(
                        given_color, expected_color,
                        "Color at ({x}, {y}) does not match"
                    );
                }
            }
        }
        // ensure iterator is exhausted
        assert!(color_iter.next().is_none());

        let iter = Transpose::from_rows(img.rows(), width, true);
        let color_iter = ColorIter::from_transpose(iter, true);

        let last_pixel = img.get_pixel(width - 2, height - 2);
        let given_last_color = color_iter.last();
        assert_eq!(last_pixel.0.get(2), given_last_color);
    }
}
