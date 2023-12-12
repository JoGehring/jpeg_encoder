use core::f32::consts::PI;
use std::ops::Mul;

use nalgebra::SMatrix;

use crate::arai::{arai_1d_column, arai_1d_row};
use crate::dct_constants::{DIRECT_LOOKUP_TABLE, MATRIX_A_MATRIX, MATRIX_A_MATRIX_TRANS};

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
pub fn direct_dct(input: &SMatrix<u16, 8, 8>) -> SMatrix<f32, 8, 8> {
    let mut output = SMatrix::from_element(0.0);
    for i in 0..8 {
        for j in 0..8 {
            let mut new_y: f32 = 0.0;
            for x in 0..8 {
                for y in 0..8 {
                    // all logic for new_y is in DIRECT_LOOKUP_TABLE
                    new_y += input[(x, y)] as f32 * DIRECT_LOOKUP_TABLE[i][j][x][y];
                }
            }
            output[(i, j)] = new_y;
        }
    }
    output
}

/// Discrete Cosine Transform on a 8x8 u16 matrix, implemented using matrix multiplication AXA^T
/// with O(n^3) complexity. Returns a 8x8 i32 matrix.
/// # Arguments
/// * `input`: The matrix to perform the DCT on.
pub fn matrix_dct(input: &SMatrix<u16, 8, 8>) -> SMatrix<f32, 8, 8> {
    let x_matrix = input.cast::<f32>();
    let y = MATRIX_A_MATRIX.mul(x_matrix).mul(MATRIX_A_MATRIX_TRANS);
    y
}

/// Perform the DCT using Arai's algorihtm.
/// This is done by first applying Arai's algorithm to all rows of the input matrix,
/// then applying it to all columns of the resulting matrix.
///
/// # Arguments
/// * `input`: The matrix to perform the DCT on.
pub fn arai_dct(input: &SMatrix<u16, 8, 8>) -> SMatrix<f32, 8, 8> {
    // first, do all rows
    let mut after_row_dct: SMatrix<f32, 8, 8> = SMatrix::zeros();
    for (i, input_row) in input.row_iter().enumerate() {
        after_row_dct.set_row(i, &arai_1d_row(&input_row.clone_owned().cast::<f32>()))
    }

    // then, do all columns
    let mut result: SMatrix<f32, 8, 8> = SMatrix::zeros();
    for (i, input_column) in after_row_dct.column_iter().enumerate() {
        result.set_column(i, &arai_1d_column(&input_column.clone_owned()))
    }

    result
}

/// Inverse Discrete Cosine Transform on a 8x8 i32 matrix, implemented directly using the standard
/// formula with O(n^4) complexity. Returns a 8x8 u16 matrix.
/// # Arguments
/// * `input`: The matrix to perform the IDCT on.
pub fn inverse_dct(input: &SMatrix<f32, 8, 8>) -> SMatrix<u16, 8, 8> {
    let mut output = SMatrix::from_element(0);
    for x in 0..8 {
        for y in 0..8 {
            let mut new_x: f32 = 0.0;
            for i in 0..8 {
                for j in 0..8 {
                    let mut product = 0.25
                        * input[(i, j)]
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
    use approx::{assert_abs_diff_eq};
    use nalgebra::SMatrix;

    use super::{arai_dct, direct_dct, inverse_dct, matrix_dct};

    #[test]
    fn test_direct_dct_from_slides() {
        test_dct_slides_vals_generic(&direct_dct);
    }

    #[test]
    fn test_matrix_dct() {
        test_dct_slides_vals_generic(&matrix_dct);
    }

    #[test]
    fn test_arai_dct_from_slides() {
        test_dct_slides_vals_generic(&arai_dct);
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
            581.25,
            -143.59541,
            56.294342,
            17.287727,
            14.750023,
            -7.179634,
            24.848614,
            -9.115171,
            -242.48978,
            132.81944,
            -47.657104,
            41.557163,
            -2.064434,
            -7.161991,
            13.281364,
            -4.1833935,
            108.385445,
            -17.976807,
            -40.036396,
            70.76318,
            -32.76093,
            12.120211,
            6.497833,
            -9.662288,
            -56.403442,
            -93.47326,
            47.70549,
            18.615627,
            -8.185273,
            7.0222178,
            6.167753,
            -1.8517237,
            -16.749977,
            9.049447,
            6.836228,
            -23.458998,
            -2.7499964,
            -9.94066,
            4.7450833,
            3.056405,
            4.0844183,
            8.789818,
            -3.572437,
            -4.8846264,
            2.4728107,
            2.1264684,
            -7.4259663,
            2.8097374,
            -9.446314,
            7.4828606,
            7.997845,
            -6.083625,
            5.181475,
            11.601294,
            2.0364275,
            -5.201282,
            -9.2168045,
            -3.5903022,
            -2.4984024,
            -2.620948,
            5.6013365,
            1.119054,
            -0.63668275,
            -1.06153,
        ];

        let y: SMatrix<f32, 8, 8> = SMatrix::from_row_iterator(y_vec.into_iter());
        let x = inverse_dct(&y);
        for i in 0..8 {
            for j in 0..8 {
                assert_abs_diff_eq!(x_expected[(i, j)], x[(i, j)], epsilon = 1);
            }
        }
    }

    fn test_dct_slides_vals_generic(dct_type: &dyn Fn(&SMatrix<u16, 8, 8>) -> SMatrix<f32, 8, 8>) {
        let x_vec = vec![
            47, 18, 13, 16, 41, 90, 47, 27, 62, 42, 35, 39, 66, 90, 41, 26, 71, 55, 56, 67, 55, 40,
            22, 39, 53, 60, 63, 50, 48, 25, 37, 87, 31, 27, 33, 27, 37, 50, 81, 147, 54, 31, 33,
            46, 58, 104, 144, 179, 76, 70, 71, 91, 118, 151, 176, 184, 102, 105, 115, 124, 135,
            168, 173, 181,
        ];
        let x = SMatrix::from_row_iterator(x_vec.into_iter());
        let y = dct_type(&x);
        let y_expected_vec = vec![
            581.25,
            -143.59541,
            56.294342,
            17.287727,
            14.750023,
            -7.179634,
            24.848614,
            -9.115171,
            -242.48978,
            132.81944,
            -47.657104,
            41.557163,
            -2.064434,
            -7.161991,
            13.281364,
            -4.1833935,
            108.385445,
            -17.976807,
            -40.036396,
            70.76318,
            -32.76093,
            12.120211,
            6.497833,
            -9.662288,
            -56.403442,
            -93.47326,
            47.70549,
            18.615627,
            -8.185273,
            7.0222178,
            6.167753,
            -1.8517237,
            -16.749977,
            9.049447,
            6.836228,
            -23.458998,
            -2.7499964,
            -9.94066,
            4.7450833,
            3.056405,
            4.0844183,
            8.789818,
            -3.572437,
            -4.8846264,
            2.4728107,
            2.1264684,
            -7.4259663,
            2.8097374,
            -9.446314,
            7.4828606,
            7.997845,
            -6.083625,
            5.181475,
            11.601294,
            2.0364275,
            -5.201282,
            -9.2168045,
            -3.5903022,
            -2.4984024,
            -2.620948,
            5.6013365,
            1.119054,
            -0.63668275,
            -1.06153,
        ];
        let y_expected: SMatrix<f32, 8, 8> = SMatrix::from_row_iterator(y_expected_vec.into_iter());

        for i in 0..8 {
            for j in 0..8 {
                assert_abs_diff_eq!(y_expected[(i, j)], y[(i, j)], epsilon = 0.01);
            }
        }
    }
}
