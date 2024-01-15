use std::ops::Mul;

use nalgebra::SMatrix;

use crate::arai::{arai_1d_column, arai_1d_row};
use crate::dct_constants::{DIRECT_LOOKUP_TABLE, MATRIX_A_MATRIX, MATRIX_A_MATRIX_TRANS};

#[allow(dead_code)]
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
pub fn direct_dct(input: &mut SMatrix<f32, 8, 8>) {
    let input_before = input.clone();
    for i in 0..8 {
        for j in 0..8 {
            let mut new_y: f32 = 0.0;
            for x in 0..8 {
                for y in 0..8 {
                    // all logic for new_y is in DIRECT_LOOKUP_TABLE
                    new_y += input_before[(x, y)] * DIRECT_LOOKUP_TABLE[i][j][x][y];
                }
            }
            input[(i, j)] = new_y;
        }
    }
}

/// Discrete Cosine Transform on a 8x8 u16 matrix, implemented using matrix multiplication AXA^T
/// with O(n^3) complexity. Returns a 8x8 i32 matrix.
/// # Arguments
/// * `input`: The matrix to perform the DCT on.
pub fn matrix_dct(input: &mut SMatrix<f32, 8, 8>) {
    MATRIX_A_MATRIX.mul(*input).mul_to(&MATRIX_A_MATRIX_TRANS, input);
}

/// Perform the DCT using Arai's algorihtm.
/// This is done by first applying Arai's algorithm to all rows of the input matrix,
/// then applying it to all columns of the resulting matrix.
///
/// # Arguments
/// * `input`: The matrix to perform the DCT on.
pub fn arai_dct(input: &mut SMatrix<f32, 8, 8>) {
    // first, do all rows
    for mut input_row in input.row_iter_mut() {
        arai_1d_row(&mut input_row);
    }

    // then, do all columns
    for mut input_column in input.column_iter_mut() {
        arai_1d_column(&mut input_column);
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_abs_diff_eq;
    use nalgebra::SMatrix;

    use super::{arai_dct, direct_dct, matrix_dct};

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

    fn test_dct_slides_vals_generic(dct_type: &dyn Fn(&mut SMatrix<f32, 8, 8>)) {
        let x_vec = vec![
            47.0, 18.0, 13.0, 16.0, 41.0, 90.0, 47.0, 27.0, 62.0, 42.0, 35.0, 39.0, 66.0, 90.0,
            41.0, 26.0, 71.0, 55.0, 56.0, 67.0, 55.0, 40.0, 22.0, 39.0, 53.0, 60.0, 63.0, 50.0,
            48.0, 25.0, 37.0, 87.0, 31.0, 27.0, 33.0, 27.0, 37.0, 50.0, 81.0, 147.0, 54.0, 31.0,
            33.0, 46.0, 58.0, 104.0, 144.0, 179.0, 76.0, 70.0, 71.0, 91.0, 118.0, 151.0, 176.0,
            184.0, 102.0, 105.0, 115.0, 124.0, 135.0, 168.0, 173.0, 181.0,
        ];
        let mut x = SMatrix::from_row_iterator(x_vec.into_iter());
        dct_type(&mut x);
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
                assert_abs_diff_eq!(y_expected[(i, j)], x[(i, j)], epsilon = 0.01);
            }
        }
    }
}
