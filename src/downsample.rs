/// Down-sample a color channel of an image.
/// `a` and `b` are expected to fit the first two parts of standard subsampling notation: https://en.wikipedia.org/wiki/Chroma_subsampling
/// TODO: replace the above link with the proper RFC/place where the notation was defined
///
/// # Arguments
///
/// * `channel`: The color channel to downsample.
/// * `a`: `a` as per the standard subsampling notation.
/// * `b`: `b` as per the standard subsampling notation.
/// * `downsample_vertical`: Whether every set of two rows should also be combined into one (vertical downsampling).
///
/// # Examples
///```
/// let result_cb = downsample_channel(&self.data2, a, b, c != 0);
/// ```
pub fn downsample_channel(
    channel: &Vec<Vec<u16>>,
    a: usize,
    b: usize,
    downsample_vertical: bool,
) -> Vec<Vec<u16>> {
    let mut final_channel: Vec<Vec<u16>> = vec![];
    for y in (0..channel.len() - 1).step_by(2) {
        let lower_row = if y + 1 < channel.len() {
            &channel[y + 1]
        } else {
            &channel[y]
        };

        let (final_row, final_lower_row) =
            downsample_rows(&channel[y], &lower_row, a, b, downsample_vertical);

        final_channel.push(final_row);
        if !downsample_vertical && y + 1 < channel.len() {
            final_channel.push(final_lower_row);
        }
    }
    return final_channel;
}

/// Down-sample the row and potentially the row below it, based on the factors `a` and `b`.
/// `a` and `b` are expected to fit the first two parts of standard subsampling notation: https://en.wikipedia.org/wiki/Chroma_subsampling
/// TODO: replace the above link with the proper RFC/place where the notation was defined
///
/// If downsample_vertical is set, the two rows are combined into the first returned row, the second one is just the downsampled `row2`.
///
/// # Arguments
///
/// * `row`: The first row to downsample.
/// * `row2`: The second row to downsample. This is usually the row below `row`.
/// * `a`: `a` as per the standard subsampling notation.
/// * `b`: `b` as per the standard subsampling notation.
/// * `downsample_vertical`: Whether the two rows should also be combined into one.
///
/// # Examples
///```
/// let row1 = &vec![16, 10, 4, 4, 13, 68, 39, 74, 38, 23, 45, 13];
/// let row2 = &vec![16, 54, 4, 96, 77, 33, 18, 23, 58, 58, 5, 45];
/// let (upper_row, lower_row) = downsample_rows(row1, row2, 4, 1, false);
///```
fn downsample_rows(
    row: &Vec<u16>,
    row2: &Vec<u16>,
    a: usize,
    b: usize,
    downsample_vertical: bool,
) -> (Vec<u16>, Vec<u16>) {
    let mut final_row: Vec<u16> = vec![];
    let mut final_lower_row: Vec<u16> = vec![];

    for x in (0..(row.len())).step_by(a) {
        let upper_row_vec = copy_and_pad(row, x, a);
        let lower_row_vec = copy_and_pad(row2, x, a);

        let mut upper_subresult = downsample_segment_of_row(&upper_row_vec, a, b);
        let mut lower_subresult = downsample_segment_of_row(&lower_row_vec, a, b);

        if downsample_vertical && a != b {
            for i in 0..upper_subresult.len() {
                let vertical_avg = overflow_safe_avg(upper_subresult[i], lower_subresult[i]);
                upper_subresult[i] = vertical_avg;
                lower_subresult[i] = vertical_avg;
            }
        }
        final_row.append(&mut upper_subresult);
        final_lower_row.append(&mut lower_subresult);
    }

    return (final_row, final_lower_row);
}

/// Copy an segment of row at the given offset and length.
/// If the segment would go beyond the row's bounds, pad it using the row's last value.
///
/// # Arguments
///
/// * `row`: The vector to copy from.
/// * `offset`: The offset to start the segment at.
/// * `length`: The length of the segment.
///
/// # Examples
///
/// ```
/// let my_vec = vec![10, 20, 30, 40];
/// let segment = copy_and_pad(&my_vec, 1, 2);
/// assert_eq!(vec![20, 30], segment);
/// let segment = copy_and_pad(&my_vec, 2, 3);
/// assert_eq!(vec![30, 40, 40], segment);
/// ```
fn copy_and_pad(row: &Vec<u16>, offset: usize, length: usize) -> Vec<u16> {
    let bound = if offset + length < row.len() {
        offset + length
    } else {
        offset + (row.len() - offset)
    };
    let row = &row[offset..bound];
    let mut row_vec: Vec<u16> = vec![0; row.len()];
    row_vec.copy_from_slice(&row);
    while row_vec.len() < length {
        row_vec.push(row_vec[row_vec.len() - 1]);
    }
    return row_vec;
}

/// Down-sample the vector, based on the factors `a` and `b`.
/// `a` and `b` are expected to fit the first two parts of standard subsampling notation: https://en.wikipedia.org/wiki/Chroma_subsampling
/// TODO: replace the above link with the proper RFC/place where the notation was defined
///
/// # Arguments
///
/// * `row_segment`: The vector to downsample.
/// * `a`: `a` as per the standard subsampling notation.
/// * `b`: `b` as per the standard subsampling notation.
///
/// # Examples
///
/// ```
/// let value = downsample_segment_of_row(vec![60, 40, 30, 20], 4, 2);
/// assert_eq!(vec![50, 25], value);
/// let value = downsample_segment_of_row(vec![60, 40, 30, 20], 4, 4);
/// assert_eq!(vec![60, 40, 30, 20], value);
/// ```
fn downsample_segment_of_row(row_segment: &[u16], a: usize, b: usize) -> Vec<u16> {
    let mut subresult: Vec<u16> = vec![0; row_segment.len()];
    subresult.copy_from_slice(&row_segment);
    let mut factor = b;
    while factor != a {
        subresult = downsample_vec_by_two(subresult);
        factor *= 2;
    }
    return subresult;
}

/// Down-sample the vector and return a vector with half the size.
///
/// # Arguments
///
/// * `original_vec`: The vector to downsample.
///
/// # Examples
///
/// ```
/// let value = downsample_vec_by_two(vec![60, 40, 30, 20]);
/// assert_eq!(vec![50, 25], value);
/// ```
fn downsample_vec_by_two(original_vec: Vec<u16>) -> Vec<u16> {
    let mut new_vec: Vec<u16> = vec![];
    for i in 0..(original_vec.len() / 2 + original_vec.len() % 2) {
        let key = if 2 * i + 1 < original_vec.len() {
            2 * i + 1
        } else {
            2 * i
        };
        new_vec.push(overflow_safe_avg(
            original_vec[2 * i],
            original_vec[key],
        ));
    }
    return new_vec;
}

/// Calculate an average between two values, while accounting for overflows.
/// This works by halving the values before adding them (avoiding overflows)
/// but also checking for whether that would lose a carry due to rounding error.
/// 
/// # Arguments
/// 
/// * `value1` First value to add up.
/// * `value2` Second value to add up.
/// 
/// # Examples
/// 
/// ```
/// let result = overflow_safe_avg(65535, 65533);
/// assert_eq!(65534, result);
/// ```
fn overflow_safe_avg(value1: u16, value2: u16) -> u16 {
    let carry = u16::from(value1 % 2 == 1 && (value2 % 2 == 1));
    let value = value1 / 2 + value2 / 2;
    carry + value
}

#[cfg(test)]
mod tests {
    use super::{
        copy_and_pad, downsample_channel, downsample_rows, downsample_segment_of_row,
        downsample_vec_by_two,
    };

    #[test]
    fn test_downsample_channel_vertical() {
        let input_channel = vec![
            vec![1, 2, 3, 4],
            vec![5, 6, 7, 8],
            vec![9, 10, 11, 12],
            vec![13, 14, 15, 16],
        ];

        let expected_output: Vec<Vec<u16>> = vec![vec![3, 5], vec![11, 13]];

        let result = downsample_channel(&input_channel, 4, 2, true);

        assert_eq!(result, expected_output);
    }

    #[test]
    fn test_downsample_channel_horizontal() {
        let input_channel = vec![
            vec![1, 2, 3, 4],
            vec![5, 6, 7, 8],
            vec![9, 10, 11, 12],
            vec![13, 14, 15, 16],
        ];

        let expected_output: Vec<Vec<u16>> =
            vec![vec![1, 3], vec![5, 7], vec![9, 11], vec![13, 15]];

        let result = downsample_channel(&input_channel, 4, 2, false);

        assert_eq!(result, expected_output);
    }

    #[test]
    fn test_downsample_channel_no_change() {
        let input_channel = vec![
            vec![1, 2, 3, 4],
            vec![5, 6, 7, 8],
            vec![9, 10, 11, 12],
            vec![13, 14, 15, 16],
        ];

        let result = downsample_channel(&input_channel, 4, 4, false);

        assert_eq!(result, input_channel);
    }

    #[test]
    fn test_downsample_row_without_vertical_single() {
        let (upper_row, lower_row) = downsample_rows(
            &vec![16, 10, 4, 4, 13, 68, 39, 74, 38, 23, 45, 13],
            &vec![16, 54, 4, 96, 77, 33, 18, 23, 58, 58, 5, 45],
            4,
            1,
            false,
        );
        assert_eq!(vec![8, 48, 29], upper_row);
        assert_eq!(vec![42, 37, 41], lower_row);
    }

    #[test]
    fn test_downsample_row_with_vertical_single() {
        let (upper_row, lower_row) = downsample_rows(
            &vec![16, 10, 4, 4, 13, 68, 39, 74, 38, 23, 45, 13],
            &vec![16, 54, 4, 96, 77, 33, 18, 23, 58, 58, 5, 45],
            4,
            1,
            true,
        );
        assert_eq!(vec![25, 42, 35], upper_row);
        assert_eq!(vec![25, 42, 35], lower_row);
    }

    #[test]
    fn test_downsample_row_without_vertical_double() {
        let (upper_row, lower_row) = downsample_rows(
            &vec![16, 10, 4, 4, 13, 68, 39, 74, 38, 23, 45, 13],
            &vec![16, 54, 4, 96, 77, 33, 18, 23, 58, 58, 5, 45],
            4,
            2,
            false,
        );
        assert_eq!(vec![13, 4, 40, 56, 30, 29], upper_row);
        assert_eq!(vec![35, 50, 55, 20, 58, 25], lower_row);
    }

    #[test]
    fn test_downsample_row_with_vertical_double() {
        let (upper_row, lower_row) = downsample_rows(
            &vec![16, 10, 4, 4, 13, 68, 39, 74, 38, 23, 45, 13],
            &vec![16, 54, 4, 96, 77, 33, 18, 23, 58, 58, 5, 45],
            4,
            2,
            true,
        );
        assert_eq!(vec![24, 27, 47, 38, 44, 27], upper_row);
        assert_eq!(vec![24, 27, 47, 38, 44, 27], lower_row);
    }

    #[test]
    fn test_downsample_row_without_vertical_none() {
        let (upper_row, lower_row) = downsample_rows(
            &vec![16, 10, 4, 4, 13, 68, 39, 74, 38, 23, 45, 13],
            &vec![16, 54, 4, 96, 77, 33, 18, 23, 58, 58, 5, 45],
            4,
            4,
            false,
        );
        assert_eq!(
            vec![16, 10, 4, 4, 13, 68, 39, 74, 38, 23, 45, 13],
            upper_row
        );
        assert_eq!(
            vec![16, 54, 4, 96, 77, 33, 18, 23, 58, 58, 5, 45],
            lower_row
        );
    }

    #[test]
    fn test_downsample_row_with_vertical_none() {
        let (upper_row, lower_row) = downsample_rows(
            &vec![16, 10, 4, 4, 13, 68, 39, 74, 38, 23, 45, 13],
            &vec![16, 54, 4, 96, 77, 33, 18, 23, 58, 58, 5, 45],
            4,
            4,
            true,
        );
        assert_eq!(
            vec![16, 10, 4, 4, 13, 68, 39, 74, 38, 23, 45, 13],
            upper_row
        );
        assert_eq!(
            vec![16, 54, 4, 96, 77, 33, 18, 23, 58, 58, 5, 45],
            lower_row
        );
    }

    #[test]
    fn test_copy_and_pad_in_bounds() {
        let my_vec = vec![10, 20, 30, 40];
        let segment = copy_and_pad(&my_vec, 1, 2);
        assert_eq!(vec![20, 30], segment);
    }

    #[test]
    fn test_copy_and_pad_out_of_bounds() {
        let my_vec = vec![10, 20, 30, 40];
        let segment = copy_and_pad(&my_vec, 2, 3);
        assert_eq!(vec![30, 40, 40], segment);
    }

    #[test]
    fn test_downsample_even_segment_of_row_none() {
        let value = downsample_segment_of_row(&[15, 17, 4, 4], 4, 4);
        assert_eq!(vec![15, 17, 4, 4], value);
    }

    #[test]
    fn test_downsample_even_segment_of_row_single() {
        let value = downsample_segment_of_row(&[15, 17, 4, 4], 4, 2);
        assert_eq!(vec![16, 4], value);
    }

    #[test]
    fn test_downsample_even_segment_of_row_twice() {
        let value = downsample_segment_of_row(&[19, 21, 4, 16], 4, 1);
        assert_eq!(vec![15], value);
    }

    #[test]
    fn test_downsample_odd_segment_of_row_none() {
        let value = downsample_segment_of_row(&[20, 10, 3], 4, 4);
        assert_eq!(vec![20, 10, 3], value);
    }

    #[test]
    fn test_downsample_odd_segment_of_row_single() {
        let value = downsample_segment_of_row(&[20, 10, 3, 5, 4], 4, 2);
        assert_eq!(vec![15, 4, 4], value);
    }

    #[test]
    fn test_downsample_odd_segment_of_row_twice() {
        let value = downsample_segment_of_row(&[44, 36, 29, 31, 10], 4, 1);
        assert_eq!(vec![35, 10], value);
    }

    #[test]
    fn test_downsample_even_vec_by_two() {
        let value = downsample_vec_by_two(vec![60, 40, 30, 20]);
        assert_eq!(vec![50, 25], value);
    }

    #[test]
    fn test_downsample_odd_vec_by_two() {
        let value = downsample_vec_by_two(vec![33, 31, 20, 40, 50]);
        assert_eq!(vec![32, 30, 50], value);
    }

    #[test]
    fn test_downsample_empty_vec_by_two() {
        let value = downsample_vec_by_two(vec![]);
        let to_compare: Vec<u16> = vec![];
        assert_eq!(to_compare, value);
    }
}
