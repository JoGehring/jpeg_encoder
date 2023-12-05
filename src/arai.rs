use std::f32::consts::{PI, SQRT_2};

use lazy_static::lazy_static;
use nalgebra::SVector;

// use lazy_static rather than consts because cos() isn't a const fn
// see https://github.com/rust-lang/rust/issues/57241
lazy_static! {
    static ref ARAI_C: [f32; 8] = [
        (0f32 * PI / 16f32).cos(),
        (1f32 * PI / 16f32).cos(),
        (2f32 * PI / 16f32).cos(),
        (3f32 * PI / 16f32).cos(),
        (4f32 * PI / 16f32).cos(),
        (5f32 * PI / 16f32).cos(),
        (6f32 * PI / 16f32).cos(),
        (7f32 * PI / 16f32).cos(),
    ];
    static ref ARAI_A: [f32; 6] = [
        0f32,
        ARAI_C[4],
        ARAI_C[2] - ARAI_C[6],
        ARAI_C[4],
        ARAI_C[6] + ARAI_C[2],
        ARAI_C[6],
    ];
    static ref ARAI_S: [f32; 8] = [
        1f32 / (2f32 * SQRT_2),
        1f32 / (4f32 * ARAI_C[1]),
        1f32 / (4f32 * ARAI_C[2]),
        1f32 / (4f32 * ARAI_C[3]),
        1f32 / (4f32 * ARAI_C[4]),
        1f32 / (4f32 * ARAI_C[5]),
        1f32 / (4f32 * ARAI_C[6]),
        1f32 / (4f32 * ARAI_C[7]),
    ];
}

/// Perform the DCT using Arai's algorithm on a Vector of size 8.
/// Arai's algorithm is split up into 4 steps, corresponding to a set
/// of additions or multiplications.
/// Since everything after the first additions has to deal with floating point
/// numbers, we can't cast back to i32 until the very end.
///
/// # Arguments
/// * `input`: A vector of integers.
pub fn arai_1d(input: &SVector<i32, 8>) -> SVector<i32, 8> {
    let mut float_vector = additions_before_first_multiplication(input);
    first_multiplications(&mut float_vector);
    additions_before_second_multiplication(&mut float_vector);
    second_multiplications(&float_vector)
}

/// Perform the first few additions of the Arai DCT algorithm.
///
/// # Arguments
/// * `input`: A vector of integers.
#[inline(always)]
fn additions_before_first_multiplication(input: &SVector<i32, 8>) -> SVector<f32, 8> {
    let mut result_vector: SVector<f32, 8> = SVector::zeros();

    result_vector[0] = (input.sum()) as f32;
    result_vector[1] =
        (input[0] + input[7] + input[3] + input[4] - input[1] - input[6] - input[2] - input[5])
            as f32;
    result_vector[2] = (input[1] + input[6] - input[2] - input[5] + input[0] + input[7]
        - input[3]
        - input[4]) as f32;
    result_vector[3] = (input[0] + input[7] - input[3] - input[4]) as f32;
    result_vector[4] = (input[4] - input[3] + input[5] - input[2]) as f32;
    result_vector[5] = (input[2] - input[5] + input[1] - input[6]) as f32;
    result_vector[6] = (input[1] - input[6] + input[0] - input[7]) as f32;
    result_vector[7] = (input[0] - input[7]) as f32;

    result_vector
}

/// Perform the first set of multiplications of the Arai DCT algorithm.
///
/// # Arguments
/// * `vector`: the vector to perform the multiplications on.
#[inline(always)]
fn first_multiplications(vector: &mut SVector<f32, 8>) {
    vector[2] *= ARAI_A[1];
    let after_a5 = (-(vector[4] + vector[6])) * ARAI_A[5];
    vector[4] = after_a5 - (vector[4] * ARAI_A[2]);
    vector[5] *= ARAI_A[3];
    vector[6] = after_a5 + (vector[6] * ARAI_A[4]);
}

/// Perform the second set of additions of the Arai DCT algorithm.
///
/// # Arguments
/// * `vector`: the vector to perform the additions on.
#[inline(always)]
fn additions_before_second_multiplication(vector: &mut SVector<f32, 8>) {
    let second_before = vector[2];
    vector[2] += vector[3];
    vector[3] -= second_before;
    let mut fifth_before = vector[5];
    vector[5] += vector[7];
    vector[7] -= fifth_before;

    fifth_before = vector[5];
    vector[5] += vector[6];
    vector[6] = fifth_before - vector[6];
    let fourth_before = vector[4];
    vector[4] += vector[7];
    vector[7] -= fourth_before;
}

/// Perform the second set of multiplications of the Arai DCT algorithm.
/// Results are cast to i32 and added to a new vector, which represents
/// the result of the DCT.
///
/// # Arguments
/// * `vector`: the vector to perform the multiplications on.
#[inline(always)]
fn second_multiplications(vector: &SVector<f32, 8>) -> SVector<i32, 8> {
    let mut result: SVector<i32, 8> = SVector::zeros();
    result[0] = multiply_and_cast(vector[0], 0);
    result[1] = multiply_and_cast(vector[5], 1);
    result[2] = multiply_and_cast(vector[2], 2);
    result[3] = multiply_and_cast(vector[7], 3);
    result[4] = multiply_and_cast(vector[1], 4);
    result[5] = multiply_and_cast(vector[4], 5);
    result[6] = multiply_and_cast(vector[3], 6);
    result[7] = multiply_and_cast(vector[6], 7);
    result
}

#[inline(always)]
fn multiply_and_cast(value: f32, index: usize) -> i32 {
    (value * ARAI_S[index]).round() as i32
}

#[cfg(test)]
mod tests {
    use nalgebra::SVector;

    use crate::arai::{
        additions_before_second_multiplication, first_multiplications, second_multiplications,
    };

    use super::{additions_before_first_multiplication, arai_1d};

    fn test_arai_1d() {
        let expected_vector: Vec<i32> = vec![12728, -6442, 0, -673, 0, -201, 0, -51];
        let expected: SVector<i32, 8> = SVector::from_row_iterator(expected_vector.into_iter());

        let values: Vec<i32> = vec![1000, 2000, 3000, 4000, 5000, 6000, 7000, 8000];
        let result = arai_1d(&SVector::from_row_iterator(values.into_iter()));

        assert_eq!(expected, result);
    }

    #[test]
    fn test_arai_1d_small_values() {
        let expected_vector: Vec<i32> = vec![106, -26, 1, 56, -13, 2, 21, -10];
        let expected: SVector<i32, 8> = SVector::from_row_iterator(expected_vector.into_iter());

        let values: Vec<i32> = vec![47, 18, 13, 16, 41, 90, 47, 27];
        let result = arai_1d(&SVector::from_row_iterator(values.into_iter()));

        assert_eq!(expected, result);
    }

    #[test]
    fn test_first_additions() {
        let values_vector: Vec<i32> = vec![47, 18, 13, 16, 41, 90, 47, 27];
        let values: SVector<i32, 8> = SVector::from_row_iterator(values_vector.into_iter());

        let actual = additions_before_first_multiplication(&values);
        let expected_vector: Vec<f32> = vec![299.0, -37.0, -21.0, 17.0, 102.0, -106.0, -9.0, 20.0];
        let expected: SVector<f32, 8> = SVector::from_row_iterator(expected_vector.into_iter());
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_first_multiplications() {
        let values_vector: Vec<f32> = vec![299.0, -37.0, -21.0, 17.0, 102.0, -106.0, -9.0, 20.0];
        let mut values: SVector<f32, 8> = SVector::from_row_iterator(values_vector.into_iter());

        first_multiplications(&mut values);
        let expected_vector: Vec<f32> = vec![
            299.0, -37.0, -14.849242, 17.0, -90.791565, -74.953316, -47.348625, 20.0,
        ];
        let expected: SVector<f32, 8> = SVector::from_row_iterator(expected_vector.into_iter());
        assert_eq!(expected, values);
    }

    #[test]
    fn test_second_additions() {
        let values_vector: Vec<f32> = vec![
            299.0, -37.0, -14.849242, 17.0, -90.791565, -74.953316, -47.348625, 20.0,
        ];
        let mut values: SVector<f32, 8> = SVector::from_row_iterator(values_vector.into_iter());

        additions_before_second_multiplication(&mut values);
        let expected_vector: Vec<f32> = vec![
            299.0, -37.0, 2.1507578, 31.849243, 4.161751, -102.30194, -7.6046906, 185.74487,
        ];
        let expected: SVector<f32, 8> = SVector::from_row_iterator(expected_vector.into_iter());
        assert_eq!(expected, values);
    }

    #[test]
    fn test_second_multiplications() {
        let values_vector: Vec<f32> = vec![
            299.0, -37.0, 2.1507578, 31.849243, 4.161751, -102.30194, -7.6046906, 185.74487,
        ];
        let values: SVector<f32, 8> = SVector::from_row_iterator(values_vector.into_iter());

        let result = second_multiplications(&values);
        let expected_vector: Vec<i32> = vec![106, -26, 1, 56, -13, 2, 21, -10];
        let expected: SVector<i32, 8> = SVector::from_row_iterator(expected_vector.into_iter());

        assert_eq!(expected, result);
    }
}
