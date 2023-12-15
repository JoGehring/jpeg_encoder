use nalgebra::{Const, Matrix, RowSVector, SVector, ViewStorageMut};

use crate::dct_constants::{ARAI_A, ARAI_S};

/// Wrapper trait so we can use the same logic on both SVector and RowSVector
pub trait Vector8 {
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

impl Vector8 for Matrix<f32, Const<1>, Const<8>, ViewStorageMut<'_, f32, Const<1>, Const<8>, Const<1>, Const<8>>> {
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

    fn zeros() -> Self {
        todo!()
    }
}

impl Vector8 for Matrix<f32, Const<8>, Const<1>, ViewStorageMut<'_, f32, Const<8>, Const<1>, Const<1>, Const<8>>> {
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

    fn zeros() -> Self {
        todo!()
    }
}

/// Perform the DCT using Arai's algorithm on a row Vector of size 8.
///
/// # Arguments
/// * `input`: A vector of integers.
pub fn arai_1d_row<T: Vector8>(input: &mut T) {
    arai_1d_internal(input);
}

/// Perform the DCT using Arai's algorithm on a column Vector of size 8.
///
/// # Arguments
/// * `input`: A vector of integers.
pub fn arai_1d_column<T: Vector8>(input: &mut T) {
    arai_1d_internal(input);
}

/// Perform the DCT using Arai's algorithm on a Vector of size 8.
/// Arai's algorithm is split up into 4 steps, corresponding to a set
/// of additions or multiplications.
/// Since everything after the first additions has to deal with floating point
/// numbers, we can't cast back to i32 until the very end.
///
/// # Arguments
/// * `input`: A vector of integers.
fn arai_1d_internal<T: Vector8>(input: &mut T) {
    additions_before_first_multiplication(input);
    first_multiplications(input);
    additions_before_second_multiplication(input);
    second_multiplications(input);
}

/// Perform the first few additions of the Arai DCT algorithm.
///
/// # Arguments
/// * `input`: A vector of integers.
#[inline(always)]
fn additions_before_first_multiplication<T: Vector8>(input: &mut T) {
    let zeroth_before = input.at(0);
    let first_before = input.at(1);
    let second_before = input.at(2);
    let third_before = input.at(3);
    input.set(0, input.sum());
    input.set(1, zeroth_before + input.at(7) + input.at(3) + input.at(4)
        - input.at(1)
        - input.at(6)
        - input.at(2)
        - input.at(5));
    input.set(2,
              first_before + input.at(6) - input.at(2) - input.at(5) + zeroth_before + input.at(7)
                  - input.at(3)
                  - input.at(4));
    input.set(3, zeroth_before + input.at(7) - input.at(3) - input.at(4));
    input.set(4, input.at(4) - third_before + input.at(5) - second_before);
    input.set(5, second_before - input.at(5) + first_before - input.at(6));
    input.set(6, first_before - input.at(6) + zeroth_before - input.at(7));
    input.set(7, zeroth_before - input.at(7));
}

/// Perform the first set of multiplications of the Arai DCT algorithm.
///
/// # Arguments
/// * `vector`: the vector to perform the multiplications on.
#[inline(always)]
fn first_multiplications<T: Vector8>(vector: &mut T) {
    let second_before = vector.at(2);
    vector.set(2, second_before * ARAI_A[1]);
    let after_a5 = (-(vector.at(4) + vector.at(6))) * ARAI_A[5];
    vector.set(4, after_a5 - (vector.at(4) * ARAI_A[2]));
    vector.set(5, vector.at(5) * ARAI_A[3]);
    vector.set(6, after_a5 + (vector.at(6) * ARAI_A[4]));
}

/// Perform the second set of additions of the Arai DCT algorithm.
///
/// # Arguments
/// * `vector`: the vector to perform the additions on.
#[inline(always)]
fn additions_before_second_multiplication<T: Vector8>(vector: &mut T) {
    let second_before = vector.at(2);
    vector.set(2, vector.at(2) + vector.at(3));
    vector.set(3, vector.at(3) - second_before);
    let mut fifth_before = vector.at(5);
    vector.set(5, vector.at(5) + vector.at(7));
    vector.set(7, vector.at(7) - fifth_before);

    fifth_before = vector.at(5);
    vector.set(5, vector.at(5) + vector.at(6));
    vector.set(6, fifth_before - vector.at(6));
    let fourth_before = vector.at(4);
    vector.set(4, vector.at(4) + vector.at(7));
    vector.set(7, vector.at(7) - fourth_before);
}

/// Perform the second set of multiplications of the Arai DCT algorithm.
/// Results are cast to i32 and added to a new vector, which represents
/// the result of the DCT.
///
/// # Arguments
/// * `vector`: the vector to perform the multiplications on.
#[inline(always)]
fn second_multiplications<T: Vector8>(vector: &mut T) {
    let first_before = vector.at(1);
    let third_before = vector.at(3);
    let fourth_before = vector.at(4);
    let sixth_before = vector.at(6);
    vector.set(0, multiply::<0>(vector.at(0)));
    vector.set(1, multiply::<1>(vector.at(5)));
    vector.set(2, multiply::<2>(vector.at(2)));
    vector.set(3, multiply::<3>(vector.at(7)));
    vector.set(4, multiply::<4>(first_before));
    vector.set(5, multiply::<5>(fourth_before));
    vector.set(6, multiply::<6>(third_before));
    vector.set(7, multiply::<7>(sixth_before));
}

#[inline(always)]
fn multiply<const I: usize>(value: f32) -> f32 {
    value * ARAI_S[I]
}

#[cfg(test)]
mod tests {
    use nalgebra::{RowSVector, SVector};

    use super::{additions_before_first_multiplication, additions_before_second_multiplication, arai_1d_column, arai_1d_row, first_multiplications, second_multiplications};

    #[test]
    fn test_arai_1d_column() {
        let expected_vector: Vec<f32> = vec![
            12727.922, -6442.3228, 0.0, -673.4549, 0.0, -200.90302, 0.0, -50.702698,
        ];
        let expected: SVector<f32, 8> = SVector::from_row_iterator(expected_vector.into_iter());

        let values: Vec<f32> = vec![
            1000.0, 2000.0, 3000.0, 4000.0, 5000.0, 6000.0, 7000.0, 8000.0,
        ];
        let mut values_vec: SVector<f32, 8> = SVector::from_row_iterator(values.into_iter());
        arai_1d_column(&mut values_vec);

        assert_eq!(expected, values_vec);
    }

    #[test]
    fn test_arai_1d_column_small_values() {
        let expected_vector: Vec<f32> = vec![
            105.71246, -26.07654, 0.5819909, 55.848366, -13.081475, 1.8727386, 20.806522, -9.745093,
        ];
        let expected: SVector<f32, 8> = SVector::from_row_iterator(expected_vector.into_iter());

        let values: Vec<f32> = vec![47.0, 18.0, 13.0, 16.0, 41.0, 90.0, 47.0, 27.0];
        let mut values_vec: SVector<f32, 8> = SVector::from_row_iterator(values.into_iter());
        arai_1d_column(&mut values_vec);
        assert_eq!(expected, values_vec);
    }

    #[test]
    fn test_arai_1d_row() {
        let expected_vector: Vec<f32> = vec![
            12727.922, -6442.3228, 0.0, -673.4549, 0.0, -200.90302, 0.0, -50.702698,
        ];
        let expected: RowSVector<f32, 8> =
            RowSVector::from_row_iterator(expected_vector.into_iter());

        let values: Vec<f32> = vec![
            1000.0, 2000.0, 3000.0, 4000.0, 5000.0, 6000.0, 7000.0, 8000.0,
        ];
        let mut values_vec: RowSVector<f32, 8> = RowSVector::from_row_iterator(values.into_iter());
        arai_1d_row(&mut values_vec);

        assert_eq!(expected, values_vec);
    }

    #[test]
    fn test_arai_1d_row_small_values() {
        let expected_vector: Vec<f32> = vec![
            105.71246, -26.07654, 0.5819909, 55.848366, -13.081475, 1.8727386, 20.806522, -9.745093,
        ];
        let expected: RowSVector<f32, 8> =
            RowSVector::from_row_iterator(expected_vector.into_iter());

        let values: Vec<f32> = vec![47.0, 18.0, 13.0, 16.0, 41.0, 90.0, 47.0, 27.0];
        let mut values_vec: RowSVector<f32, 8> = RowSVector::from_row_iterator(values.into_iter());
        let result = arai_1d_row(&mut values_vec);

        assert_eq!(expected, values_vec);
    }
    #[test]
    fn test_first_additions() {
        let values_vector: Vec<f32> = vec![47.0, 18.0, 13.0, 16.0, 41.0, 90.0, 47.0, 27.0];
        let mut values: SVector<f32, 8> = SVector::from_row_iterator(values_vector.into_iter());

        additions_before_first_multiplication(&mut values);
        let expected_vector: Vec<f32> = vec![299.0, -37.0, -21.0, 17.0, 102.0, -106.0, -9.0, 20.0];
        let expected: SVector<f32, 8> = SVector::from_row_iterator(expected_vector.into_iter());
        assert_eq!(expected, values);
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
        let mut values: SVector<f32, 8> = SVector::from_row_iterator(values_vector.into_iter());

        second_multiplications::<SVector<f32, 8>>(&mut values);
        let expected_vector: Vec<f32> = vec![
            105.71246, -26.07654, 0.5819909, 55.848366, -13.081475, 1.8727386, 20.806522, -9.745093,
        ];
        let expected: SVector<f32, 8> = SVector::from_row_iterator(expected_vector.into_iter());

        assert_eq!(expected, values);
    }
}
