use core::f32;

use nalgebra::SMatrix;

/// Create a uniform quantization matrix from factor x in format 1/x
/// # Arguments
/// * `factor`: The quantization factor
pub fn uniform_q_table(factor: f32) -> SMatrix<f32, 8, 8> {
    SMatrix::from_element(1.0 / factor)
}

/// Create a quantization matrix that linearly gets higher values towards the bottom right,
/// in the format 1/x.
/// All values have global_factor applied to them, then with each step towards the bottom or right
/// growth_factor is added another time
/// (so 1 time for the top left, 2 times for the cell below that, 3 times for the cell to the right of that)
pub fn linear_q_table(global_factor: f32, growth_factor: f32) -> SMatrix<f32, 8, 8> {
    let mut matrix = uniform_q_table(global_factor);

    for i in 0..8 {
        for j in 0..8 {
            matrix[(i, j)] /= (i + 1) as f32 * (j + 1) as f32 * growth_factor;
        }
    }

    matrix
}

/// Create a quantization table with increasingly higher values in "boxes".
/// Each value gets assigned the global_factor first.
/// Then the matrix is split up into square submatrices of size box_size and
/// each box is applied box_growth n times, with n being the box' x/y position.
/// (so 0 for the top left, 1 for the box below that, 2 for the box to the right of that)
pub fn box_q_table(global_factor: f32, box_size: usize, box_growth: f32) -> SMatrix<f32, 8, 8> {
    SMatrix::from_fn(|x, y| {
        let mut fac = global_factor;
        fac += (x / box_size + y / box_size) as f32 * box_growth;
        1.0 / fac
    })
}

/// Quantize the given matrix by multiplying it component-wise with
/// the quantization table with format 1/x. The condition in the map only
/// applies to exact 0.5 values, e.g. in test_quatization_from_slides, value 25.0 and
/// ensures a 0 instead of 1 for this border case for better compression
/// # Arguments
/// * `data`: The matrix to perform the quantization on
/// * `q_table`: The quantization matrix with quantization factor x in format 1/x
pub fn quantize(data: &mut SMatrix<f32, 8, 8>, q_table: &SMatrix<f32, 8, 8>) {
    data.component_mul_assign(q_table);
    data.apply(|value| {
        if *value == 0.5 {
            *value = 0.0;
        } else {
            *value = value.round();
        }
    });
}

/// Zigzag sample the given data.
/// The sampling is hardcoded for simplicity reasons.
/// # Arguments
/// * `data`: The matrix to zigzag sample.
pub fn sample_zigzag<T: Copy>(data: &SMatrix<T, 8, 8>) -> [T; 64] {
    [
        data[(0, 0)],
        data[(0, 1)],
        data[(1, 0)],
        data[(2, 0)],
        data[(1, 1)],
        data[(0, 2)],
        data[(0, 3)],
        data[(1, 2)],
        data[(2, 1)],
        data[(3, 0)],
        data[(4, 0)],
        data[(3, 1)],
        data[(2, 2)],
        data[(1, 3)],
        data[(0, 4)],
        data[(0, 5)],
        data[(1, 4)],
        data[(2, 3)],
        data[(3, 2)],
        data[(4, 1)],
        data[(5, 0)],
        data[(6, 0)],
        data[(5, 1)],
        data[(4, 2)],
        data[(3, 3)],
        data[(2, 4)],
        data[(1, 5)],
        data[(0, 6)],
        data[(0, 7)],
        data[(1, 6)],
        data[(2, 5)],
        data[(3, 4)],
        data[(4, 3)],
        data[(5, 2)],
        data[(6, 1)],
        data[(7, 0)],
        data[(7, 1)],
        data[(6, 2)],
        data[(5, 3)],
        data[(4, 4)],
        data[(3, 5)],
        data[(2, 6)],
        data[(1, 7)],
        data[(2, 7)],
        data[(3, 6)],
        data[(4, 5)],
        data[(5, 4)],
        data[(6, 3)],
        data[(7, 2)],
        data[(7, 3)],
        data[(6, 4)],
        data[(5, 5)],
        data[(4, 6)],
        data[(3, 7)],
        data[(4, 7)],
        data[(5, 6)],
        data[(6, 5)],
        data[(7, 4)],
        data[(7, 5)],
        data[(6, 6)],
        data[(5, 7)],
        data[(6, 7)],
        data[(7, 6)],
        data[(7, 7)],
    ]
}

#[cfg(test)]
mod test {
    use nalgebra::SMatrix;

    use super::{quantize, sample_zigzag, uniform_q_table};

    #[test]
    fn test_quantization_from_slides() {
        let x_vec = vec![
            581.0, -144.0, 56.0, 17.0, 15.0, -7.0, 25.0, -9.0, -242.0, 133.0, -48.0, 42.0, -2.0,
            -7.0, 13.0, -4.0, 108.0, -18.0, -40.0, 71.0, -33.0, 12.0, 6.0, -10.0, -56.0, -93.0,
            48.0, 19.0, -8.0, 7.0, 6.0, -2.0, -17.0, 9.0, 7.0, -23.0, -3.0, -10.0, 5.0, 3.0, 4.0,
            9.0, -4.0, -5.0, 2.0, 2.0, -7.0, 3.0, -9.0, 7.0, 8.0, -6.0, 5.0, 12.0, 2.0, -5.0, -9.0,
            -4.0, -2.0, -3.0, 6.0, 1.0, -1.0, -1.0,
        ];
        let mut x: SMatrix<f32, 8, 8> = SMatrix::from_row_iterator(x_vec.into_iter());
        let y_vec = vec![
            12.0, -3.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, -5.0, 3.0, -1.0, 1.0, 0.0, 0.0, 0.0, 0.0,
            2.0, 0.0, -1.0, 1.0, -1.0, 0.0, 0.0, 0.0, -1.0, -2.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
        ];
        let expected: SMatrix<f32, 8, 8> = SMatrix::from_row_iterator(y_vec.into_iter());
        let q_table = uniform_q_table(50.0);
        quantize(&mut x, &q_table);
        assert_eq!(expected, x);
    }

    #[test]
    fn test_zigzag_sampling_slides() {
        let expected_vec = vec![
            12, -3, 1, 0, 0, 0, 0, 0, -5, 3, -1, 1, 0, 0, 0, 0, 2, 0, -1, 1, -1, 0, 0, 0, -1, -2,
            1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];
        let expected_matrix: SMatrix<i32, 8, 8> =
            SMatrix::from_row_iterator(expected_vec.into_iter());
        let expected: [i32; 64] = [
            12, -3, -5, 2, 3, 1, 0, -1, 0, -1, 0, -2, -1, 1, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, -1,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];
        let result = sample_zigzag(&expected_matrix);
        assert_eq!(expected, result);
    }

    #[test]
    fn test_zigzag_sampling_sequence() {
        let mut expected_vec: Vec<i32> = Vec::with_capacity(64);
        for i in 0..=63 {
            expected_vec.push(i);
        }
        let expected_matrix: SMatrix<i32, 8, 8> =
            SMatrix::from_row_iterator(expected_vec.into_iter());
        let expected: [i32; 64] = [
            0, 1, 8, 16, 9, 2, 3, 10, 17, 24, 32, 25, 18, 11, 4, 5, 12, 19, 26, 33, 40, 48, 41, 34,
            27, 20, 13, 6, 7, 14, 21, 28, 35, 42, 49, 56, 57, 50, 43, 36, 29, 22, 15, 23, 30, 37,
            44, 51, 58, 59, 52, 45, 38, 31, 39, 46, 53, 60, 61, 54, 47, 55, 62, 63,
        ];
        let result = sample_zigzag(&expected_matrix);
        assert_eq!(expected, result);
    }
}
