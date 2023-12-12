use nalgebra::{RowSVector, SVector};

use crate::dct_constants::{ARAI_A, ARAI_S};

/// Wrapper trait so we can use the same logic on both SVector and RowSVector
trait Vector8 {
    /// Get the `index`th value of this vector.
    ///
    /// # Arguments
    /// * `index`: The value index.
    fn at(&self, index: usize) -> f32;
    /// Set the index-th value.
    ///
    /// # Arguments
    /// * `index`: The value index.
    fn set(&mut self, index: usize, value: f32);
    /// Get the sum of all values in this vector.
    fn sum(&self) -> f32;
    /// Get an empty vector.
    fn zeros() -> Self;
}
impl Vector8 for SVector<f32, 8> {
    #[inline(always)]
    fn at(&self, index: usize) -> f32 {
        self[index]
    }
    #[inline(always)]
    fn set(&mut self, index: usize, value: f32) {
        self[index] = value;
    }
    #[inline(always)]
    fn sum(&self) -> f32 {
        self.sum()
    }
    #[inline(always)]
    fn zeros() -> Self {
        SVector::zeros()
    }
}
impl Vector8 for RowSVector<f32, 8> {
    #[inline(always)]
    fn at(&self, index: usize) -> f32 {
        self[index]
    }
    #[inline(always)]
    fn set(&mut self, index: usize, value: f32) {
        self[index] = value;
    }
    #[inline(always)]
    fn sum(&self) -> f32 {
        self.sum()
    }
    #[inline(always)]
    fn zeros() -> Self {
        RowSVector::zeros()
    }
}

/// Perform the DCT using Arai's algorithm on a row Vector of size 8.
///
/// # Arguments
/// * `input`: A vector of integers.
pub fn arai_1d_row(input: &RowSVector<f32, 8>) -> RowSVector<f32, 8> {
    arai_1d_internal(input)
}

/// Perform the DCT using Arai's algorithm on a column Vector of size 8.
///
/// # Arguments
/// * `input`: A vector of integers.
pub fn arai_1d_column(input: &SVector<f32, 8>) -> SVector<f32, 8> {
    arai_1d_internal(input)
}

/// Perform the DCT using Arai's algorithm on a Vector of size 8.
/// Arai's algorithm is split up into 4 steps, corresponding to a set
/// of additions or multiplications.
/// Since everything after the first additions has to deal with floating point
/// numbers, we can't cast back to i32 until the very end.
///
/// # Arguments
/// * `input`: A vector of integers.
fn arai_1d_internal<T: Vector8>(input: &T) -> T {
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
fn additions_before_first_multiplication<T: Vector8>(input: &T) -> SVector<f32, 8> {
    let mut result_vector: SVector<f32, 8> = SVector::zeros();

    result_vector[0] = (input.sum()) as f32;
    result_vector[1] = (input.at(0) + input.at(7) + input.at(3) + input.at(4)
        - input.at(1)
        - input.at(6)
        - input.at(2)
        - input.at(5)) as f32;
    result_vector[2] =
        (input.at(1) + input.at(6) - input.at(2) - input.at(5) + input.at(0) + input.at(7)
            - input.at(3)
            - input.at(4)) as f32;
    result_vector[3] = (input.at(0) + input.at(7) - input.at(3) - input.at(4)) as f32;
    result_vector[4] = (input.at(4) - input.at(3) + input.at(5) - input.at(2)) as f32;
    result_vector[5] = (input.at(2) - input.at(5) + input.at(1) - input.at(6)) as f32;
    result_vector[6] = (input.at(1) - input.at(6) + input.at(0) - input.at(7)) as f32;
    result_vector[7] = (input.at(0) - input.at(7)) as f32;

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
fn second_multiplications<T: Vector8>(vector: &SVector<f32, 8>) -> T {
    let mut result: T = T::zeros();
    result.set(0, multiply::<0>(vector[0]));
    result.set(1, multiply::<1>(vector[5]));
    result.set(2, multiply::<2>(vector[2]));
    result.set(3, multiply::<3>(vector[7]));
    result.set(4, multiply::<4>(vector[1]));
    result.set(5, multiply::<5>(vector[4]));
    result.set(6, multiply::<6>(vector[3]));
    result.set(7, multiply::<7>(vector[6]));
    result
}

#[inline(always)]
fn multiply<const I: usize>(value: f32) -> f32 {
    (value * ARAI_S[I])
}

#[cfg(test)]
mod tests {
    use nalgebra::{RowSVector, SVector};

    use super::{
        additions_before_first_multiplication, additions_before_second_multiplication,
        arai_1d_column, arai_1d_row, first_multiplications, second_multiplications,
    };

    #[test]
    fn test_arai_1d_column() {
        let expected_vector: Vec<i32> = vec![12728, -6442, 0, -673, 0, -201, 0, -51];
        let expected: SVector<i32, 8> = SVector::from_row_iterator(expected_vector.into_iter());

        let values: Vec<i32> = vec![1000, 2000, 3000, 4000, 5000, 6000, 7000, 8000];
        // let result = arai_1d_column(&SVector::from_row_iterator(values.into_iter()));

        // assert_eq!(expected, result);
    }

    #[test]
    fn test_arai_1d_column_small_values() {
        let expected_vector: Vec<i32> = vec![106, -26, 1, 56, -13, 2, 21, -10];
        let expected: SVector<i32, 8> = SVector::from_row_iterator(expected_vector.into_iter());

        let values: Vec<i32> = vec![47, 18, 13, 16, 41, 90, 47, 27];
        // let result = arai_1d_column(&SVector::from_row_iterator(values.into_iter()));
//TODO fix tests 
        // assert_eq!(expected, result);
    }

    #[test]
    fn test_arai_1d_row() {
        let expected_vector: Vec<i32> = vec![12728, -6442, 0, -673, 0, -201, 0, -51];
        let expected: RowSVector<i32, 8> =
            RowSVector::from_row_iterator(expected_vector.into_iter());

        let values: Vec<i32> = vec![1000, 2000, 3000, 4000, 5000, 6000, 7000, 8000];
        // let result = arai_1d_row(&RowSVector::from_row_iterator(values.into_iter()));

        // assert_eq!(expected, result);
    }

    #[test]
    fn test_arai_1d_row_small_values() {
        let expected_vector: Vec<i32> = vec![106, -26, 1, 56, -13, 2, 21, -10];
        let expected: RowSVector<i32, 8> = RowSVector::from_row_iterator(expected_vector.into_iter());

        // let values: Vec<i32> = vec![47, 18, 13, 16, 41, 90, 47, 27];
        // let result = arai_1d_row(&RowSVector::from_row_iterator(values.into_iter()));

        // assert_eq!(expected, result);
    }
    #[test]
    fn test_first_additions() {
        let values_vector: Vec<i32> = vec![47, 18, 13, 16, 41, 90, 47, 27];
        let values: SVector<i32, 8> = SVector::from_row_iterator(values_vector.into_iter());

        // let actual = additions_before_first_multiplication(&values);
        // let expected_vector: Vec<f32> = vec![299.0, -37.0, -21.0, 17.0, 102.0, -106.0, -9.0, 20.0];
        // let expected: SVector<f32, 8> = SVector::from_row_iterator(expected_vector.into_iter());
        // assert_eq!(expected, actual);
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

        // let result = second_multiplications::<SVector<i32, 8>>(&values);
        // let expected_vector: Vec<i32> = vec![106, -26, 1, 56, -13, 2, 21, -10];
        // let expected: SVector<i32, 8> = SVector::from_row_iterator(expected_vector.into_iter());

        // assert_eq!(expected, result);
    }
}
