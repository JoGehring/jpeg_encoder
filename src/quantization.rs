use core::f32;

use nalgebra::SMatrix;
/// Create an uniform quantization matrix from factor x in format 1/x
/// # Arguments
/// * `factor`: The quantization factor
fn create_uniform_q_table(factor: f32) -> SMatrix<f32, 8, 8> {
    SMatrix::from_element(1.0/factor)
}

/// Quantize the given matrix by multiplying it component-wise with
/// the quantization table with format 1/x. The condition in the map only
/// applies to exact 0.5 values, e.g. in test_quatization_from_slides, value 25.0 and
/// ensures a 0 instead of 1 for this border case for better compression
/// # Arguments
/// * `data`: The matrix to perform the quantization on
/// * `q_table`: The quantization matrix with quantization factor x in format 1/x
fn quantize(data: &SMatrix<f32, 8, 8>, q_table: &SMatrix<f32, 8, 8>) -> SMatrix<i16, 8, 8> {
    let result = data.component_mul(q_table);
    result.map(|value| if value == 0.5{0.0} else {value.round()}).try_cast::<i16>().unwrap()
}

#[cfg(test)]
mod test {
    use nalgebra::SMatrix;

    use super::{create_uniform_q_table, quantize};

    #[test]
    fn test_quatization_from_slides() {
           let x_vec = vec![
            581.0, -144.0, 56.0, 17.0, 15.0, -7.0, 25.0, -9.0,
               -242.0, 133.0, -48.0, 42.0, -2.0, -7.0, 13.0, -4.0,
               108.0, -18.0, -40.0, 71.0, -33.0, 12.0, 6.0, -10.0,
               -56.0, -93.0, 48.0, 19.0, -8.0, 7.0, 6.0, -2.0,
               -17.0, 9.0, 7.0, -23.0, -3.0, -10.0, 5.0, 3.0,
               4.0, 9.0, -4.0, -5.0, 2.0, 2.0, -7.0, 3.0,
               -9.0, 7.0, 8.0, -6.0, 5.0, 12.0, 2.0, -5.0,
               -9.0, -4.0, -2.0, -3.0, 6.0, 1.0, -1.0, -1.0
           ];
        let x: SMatrix<f32, 8, 8> = SMatrix::from_row_iterator(x_vec.into_iter());
        let y_vec = vec![
            12, -3, 1, 0, 0, 0, 0,0,
            -5, 3, -1, 1, 0, 0, 0,0,
            2, 0, -1, 1, -1, 0, 0,0,
            -1, -2, 1, 0, 0, 0, 0,0, 
            0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 
            0, 0, 0, 0, 0, 0, 0, 0, 
            0, 0, 0, 0, 0, 0, 0, 0, 
        ];
        let expected: SMatrix<i16, 8, 8> = SMatrix::from_row_iterator(y_vec.into_iter());
        let q_table = create_uniform_q_table(50.0);
        let result = quantize(&x, &q_table);
        assert_eq!(expected, result);
 
    }
}
