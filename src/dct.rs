use core::f32::consts::PI;
use lazy_static::lazy_static;
use std::{f32::consts::SQRT_2, ops::Mul};

use nalgebra::SMatrix;

use crate::arai::arai_1d;

const MATRIX_C0: f32 = 1.0 / SQRT_2;
lazy_static! {
    static ref MATRIX_SQRT_CONST: f32 = 0.25f32.sqrt();
}

pub enum DCTMode {
    Direct,
    Matrix,
    Arai,
}

impl std::fmt::Display for DCTMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                DCTMode::Direct => "direct",
                DCTMode::Matrix => "matrix",
                DCTMode::Arai => "arai",
            }
        )
    }
}

/// Discrete Cosine Transform on a 8x8 u16 matrix, implemented directly using the standard
/// formula with O(n^4) complexity. Returns a 8x8 i32 matrix.
/// # Arguments
/// * `input`: The matrix to perform the DCT on.
pub fn direct_dct(input: &SMatrix<u16, 8, 8>) -> SMatrix<i32, 8, 8> {
    let mut output = SMatrix::from_element(0);
    for i in 0..8 {
        for j in 0..8 {
            let mut new_y: f32 = 0.0;
            for x in 0..8 {
                for y in 0..8 {
                    new_y += input[(x, y)] as f32
                        * (((2 * x + 1) as f32 * i as f32 * PI) / 16.0).cos()
                        * (((2 * y + 1) as f32 * j as f32 * PI) / 16.0).cos();
                }
            }
            new_y *= 0.25;
            if i == 0 {
                new_y *= 1.0 / 2_f32.sqrt()
            }
            if j == 0 {
                new_y *= 1.0 / 2_f32.sqrt()
            }
            output[(i, j)] = new_y.round() as i32;
        }
    }
    output
}

/// Discrete Cosine Transform on a 8x8 u16 matrix, implemented using matrix multiplication AXA^T
/// with O(n^3) complexity. Returns a 8x8 i32 matrix.
/// # Arguments
/// * `input`: The matrix to perform the DCT on.
pub fn matrix_dct(input: &SMatrix<u16, 8, 8>) -> SMatrix<i32, 8, 8> {
    let mut a_matrix: SMatrix<f32, 8, 8> = SMatrix::from_element(0.0);
    let mut x_matrix: SMatrix<f32, 8, 8> = SMatrix::from_element(0.0);
    for k in 0..8 {
        for n in 0..8 {
            let cos_val = (((2 * n + 1) * k) as f32 * PI / 16.0f32).cos();
            a_matrix[(k, n)] = cos_val * *MATRIX_SQRT_CONST * if k == 0 { MATRIX_C0 } else { 1.0 };
            x_matrix[(k, n)] = input[(k, n)] as f32;
        }
    }
    let y = a_matrix.mul(x_matrix).mul(a_matrix.transpose());
    y.map(|x| x.round() as i32)
}

/// Perform the DCT using Arai's algorihtm.
/// This is done by first applying Arai's algorithm to all rows of the input matrix,
/// then applying it to all columns of the resulting matrix.
///
/// # Arguments
/// * `input`: The matrix to perform the DCT on.
pub fn arai_dct(input: &SMatrix<u16, 8, 8>) -> SMatrix<i32, 8, 8> {
    // first, do all rows
    let mut after_row_dct: SMatrix<i32, 8, 8> = SMatrix::zeros();
    for (i, input_row) in input.row_iter().enumerate() {
        after_row_dct.set_row(
            i,
            &arai_1d(&input_row.clone_owned().cast::<i32>().transpose()).transpose(),
        )
    }

    // then, do all columns
    let mut result: SMatrix<i32, 8, 8> = SMatrix::zeros();
    for (i, input_column) in after_row_dct.column_iter().enumerate() {
        result.set_column(i, &arai_1d(&input_column.clone_owned()))
    }

    result
}

/// Inverse Discrete Cosine Transform on a 8x8 i32 matrix, implemented directly using the standard
/// formula with O(n^4) complexity. Returns a 8x8 u16 matrix.
/// # Arguments
/// * `input`: The matrix to perform the IDCT on.
pub fn inverse_dct(input: &SMatrix<i32, 8, 8>) -> SMatrix<u16, 8, 8> {
    let mut output = SMatrix::from_element(0);
    for x in 0..8 {
        for y in 0..8 {
            let mut new_x: f32 = 0.0;
            for i in 0..8 {
                for j in 0..8 {
                    let mut product = 0.25
                        * input[(i, j)] as f32
                        * (((2 * x + 1) as f32 * i as f32 * PI) / 16.0).cos()
                        * (((2 * y + 1) as f32 * j as f32 * PI) / 16.0).cos();
                    if i == 0 {
                        product *= 1.0 / 2_f32.sqrt()
                    }
                    if j == 0 {
                        product *= 1.0 / 2_f32.sqrt()
                    }
                    new_x += product;
                }
            }
            output[(x, y)] = new_x.round() as u16;
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use nalgebra::SMatrix;

    use super::{arai_dct, direct_dct, inverse_dct, matrix_dct};

    #[test]
    fn test_direct_dct_from_slides() {
        test_dct_slides_vals_generic(&direct_dct);
    }

    #[test]
    fn test_arai_dct_from_slides() {
        // slightly different values compared to the test above, due to rounding errors/differences
        // different approach would be the usage of a testing crate (e.g. 'approx'), which checks for
        // given deltas
        let x_vec = vec![
            47, 18, 13, 16, 41, 90, 47, 27, 62, 42, 35, 39, 66, 90, 41, 26, 71, 55, 56, 67, 55, 40,
            22, 39, 53, 60, 63, 50, 48, 25, 37, 87, 31, 27, 33, 27, 37, 50, 81, 147, 54, 31, 33,
            46, 58, 104, 144, 179, 76, 70, 71, 91, 118, 151, 176, 184, 102, 105, 115, 124, 135,
            168, 173, 181,
        ];
        let x = SMatrix::from_row_iterator(x_vec.into_iter());
        let y = arai_dct(&x);
        let y_expected_vec = vec![
            581, -144, 56, 17, 15, -7, 25, -8, -242, 133, -47, 42, -2, -7, 13, -5, 108, -18, -40,
            71, -33, 12, 7, -10, -57, -93, 48, 19, -8, 7, 6, -2, -16, 9, 7, -23, -3, -10, 5, 3, 4,
            9, -3, -5, 2, 2, -7, 2, -10, 8, 8, -6, 5, 12, 2, -5, -9, -4, -3, -2, 5, 1, -1, -1,
        ];
        let y_expected: SMatrix<i32, 8, 8> = SMatrix::from_row_iterator(y_expected_vec.into_iter());

        assert_eq!(y_expected, y);
    }

    #[test]
    fn test_inverse_dct_from_slides() {
        // slightly different values compared to the test above, due to rounding errors/differences
        // different approach would be the usage of a testing crate (e.g. 'approx'), which checks for
        // given deltas
        let x_vec = vec![
            47, 18, 13, 16, 41, 90, 47, 27, 62, 42, 35, 39, 66, 90, 41, 26, 71, 55, 56, 67, 55, 40,
            23, 39, 53, 59, 64, 50, 48, 25, 37, 87, 31, 27, 33, 27, 37, 50, 81, 148, 54, 31, 33,
            46, 58, 104, 144, 179, 76, 70, 71, 91, 118, 151, 176, 184, 101, 105, 115, 124, 135,
            168, 173, 181,
        ];
        let x_expected: SMatrix<u16, 8, 8> = SMatrix::from_row_iterator(x_vec.into_iter());
        let y_vec = vec![
            581, -144, 56, 17, 15, -7, 25, -9, -242, 133, -48, 42, -2, -7, 13, -4, 108, -18, -40,
            71, -33, 12, 6, -10, -56, -93, 48, 19, -8, 7, 6, -2, -17, 9, 7, -23, -3, -10, 5, 3, 4,
            9, -4, -5, 2, 2, -7, 3, -9, 7, 8, -6, 5, 12, 2, -5, -9, -4, -2, -3, 6, 1, -1, -1,
        ];
        let y: SMatrix<i32, 8, 8> = SMatrix::from_row_iterator(y_vec.into_iter());
        let x = inverse_dct(&y);
        assert_eq!(x_expected, x);
    }

    #[test]
    fn test_matrix_dct() {
        test_dct_slides_vals_generic(&matrix_dct);
    }

    fn test_dct_slides_vals_generic(dct_type: &dyn Fn(&SMatrix<u16, 8, 8>) -> SMatrix<i32, 8, 8>) {
        let x_vec = vec![
            47, 18, 13, 16, 41, 90, 47, 27, 62, 42, 35, 39, 66, 90, 41, 26, 71, 55, 56, 67, 55, 40,
            22, 39, 53, 60, 63, 50, 48, 25, 37, 87, 31, 27, 33, 27, 37, 50, 81, 147, 54, 31, 33,
            46, 58, 104, 144, 179, 76, 70, 71, 91, 118, 151, 176, 184, 102, 105, 115, 124, 135,
            168, 173, 181,
        ];
        let x = SMatrix::from_row_iterator(x_vec.into_iter());
        let y = dct_type(&x);
        let y_expected_vec = vec![
            581, -144, 56, 17, 15, -7, 25, -9, -242, 133, -48, 42, -2, -7, 13, -4, 108, -18, -40,
            71, -33, 12, 6, -10, -56, -93, 48, 19, -8, 7, 6, -2, -17, 9, 7, -23, -3, -10, 5, 3, 4,
            9, -4, -5, 2, 2, -7, 3, -9, 7, 8, -6, 5, 12, 2, -5, -9, -4, -2, -3, 6, 1, -1, -1,
        ];
        let y_expected: SMatrix<i32, 8, 8> = SMatrix::from_row_iterator(y_expected_vec.into_iter());

        assert_eq!(y_expected, y);
    }
}
