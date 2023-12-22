use core::f32;

use nalgebra::SMatrix;
/// Create an uniform quantization matrix from factor x in format 1/x
/// # Arguments
/// * `factor`: The quantization factor
fn create_uniform_q_table(factor: f32) -> SMatrix<f32, 8, 8> {
    SMatrix::from_element(1.0 / factor)
}

/// Quantize the given matrix by multiplying it component-wise with
/// the quantization table with format 1/x. The condition in the map only
/// applies to exact 0.5 values, e.g. in test_quatization_from_slides, value 25.0 and
/// ensures a 0 instead of 1 for this border case for better compression
/// # Arguments
/// * `data`: The matrix to perform the quantization on
/// * `q_table`: The quantization matrix with quantization factor x in format 1/x
fn quantize(data: &SMatrix<f32, 8, 8>, q_table: &SMatrix<f32, 8, 8>) -> SMatrix<i32, 8, 8> {
    let result = data.component_mul(q_table);
    result
        .map(|value| if value == 0.5 { 0.0 } else { value.round() })
        .try_cast::<i32>()
        .unwrap()
}

fn sample_zigzag(data: &SMatrix<i32, 8, 8>) -> [i32; 64] {
    let mut result = [0; 64];
    result[0] = data[(0, 0)];
    result[1] = data[(0, 1)];
    result[2] = data[(1, 0)];
    result[3] = data[(2, 0)];
    result[4] = data[(1, 1)];
    result[5] = data[(0, 2)];
    result[6] = data[(0, 3)];
    result[7] = data[(1, 2)];
    result[8] = data[(2, 1)];
    result[9] = data[(3, 0)];
    result[10] = data[(4, 0)];
    result[11] = data[(3, 1)];
    result[12] = data[(2, 2)];
    result[13] = data[(1, 3)];
    result[14] = data[(0, 4)];
    result[15] = data[(0, 5)];
    result[16] = data[(1, 4)];
    result[17] = data[(2, 3)];
    result[18] = data[(3, 2)];
    result[19] = data[(4, 1)];
    result[20] = data[(5, 0)];
    result[21] = data[(6, 0)];
    result[22] = data[(5, 1)];
    result[23] = data[(4, 2)];
    result[24] = data[(3, 3)];
    result[25] = data[(2, 4)];
    result[26] = data[(1, 5)];
    result[27] = data[(0, 6)];
    result[28] = data[(0, 7)];
    result[29] = data[(1, 6)];
    result[30] = data[(2, 5)];
    result[31] = data[(3, 4)];
    result[32] = data[(4, 3)];
    result[33] = data[(5, 2)];
    result[34] = data[(6, 1)];
    result[35] = data[(7, 0)];
    result[36] = data[(7, 1)];
    result[37] = data[(6, 2)];
    result[38] = data[(5, 3)];
    result[39] = data[(4, 4)];
    result[40] = data[(3, 5)];
    result[41] = data[(2, 6)];
    result[42] = data[(1, 7)];
    result[43] = data[(2, 7)];
    result[44] = data[(3, 6)];
    result[45] = data[(4, 5)];
    result[46] = data[(5, 4)];
    result[47] = data[(6, 3)];
    result[48] = data[(7, 2)];
    result[49] = data[(7, 3)];
    result[50] = data[(6, 4)];
    result[51] = data[(5, 5)];
    result[52] = data[(4, 6)];
    result[53] = data[(3, 7)];
    result[54] = data[(4, 7)];
    result[55] = data[(5, 6)];
    result[56] = data[(6, 5)];
    result[57] = data[(7, 4)];
    result[58] = data[(7, 5)];
    result[59] = data[(6, 6)];
    result[60] = data[(5, 7)];
    result[61] = data[(6, 7)];
    result[62] = data[(7, 6)];
    result[63] = data[(7, 7)];
    result
}

#[cfg(test)]
mod test {
    use nalgebra::SMatrix;

    use super::{create_uniform_q_table, quantize, sample_zigzag};

    #[test]
    fn test_quatization_from_slides() {
        let x_vec = vec![
            581.0, -144.0, 56.0, 17.0, 15.0, -7.0, 25.0, -9.0, -242.0, 133.0, -48.0, 42.0, -2.0,
            -7.0, 13.0, -4.0, 108.0, -18.0, -40.0, 71.0, -33.0, 12.0, 6.0, -10.0, -56.0, -93.0,
            48.0, 19.0, -8.0, 7.0, 6.0, -2.0, -17.0, 9.0, 7.0, -23.0, -3.0, -10.0, 5.0, 3.0, 4.0,
            9.0, -4.0, -5.0, 2.0, 2.0, -7.0, 3.0, -9.0, 7.0, 8.0, -6.0, 5.0, 12.0, 2.0, -5.0, -9.0,
            -4.0, -2.0, -3.0, 6.0, 1.0, -1.0, -1.0,
        ];
        let x: SMatrix<f32, 8, 8> = SMatrix::from_row_iterator(x_vec.into_iter());
        let y_vec = vec![
            12, -3, 1, 0, 0, 0, 0, 0, -5, 3, -1, 1, 0, 0, 0, 0, 2, 0, -1, 1, -1, 0, 0, 0, -1, -2,
            1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];
        let expected: SMatrix<i32, 8, 8> = SMatrix::from_row_iterator(y_vec.into_iter());
        let q_table = create_uniform_q_table(50.0);
        let result = quantize(&x, &q_table);
        assert_eq!(expected, result);
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
