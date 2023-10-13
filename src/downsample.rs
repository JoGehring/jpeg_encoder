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
///
/// TODO
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
///
/// TODO
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

        if downsample_vertical {
            for i in 0..(upper_subresult.len() - 1) {
                upper_subresult[i] = (upper_subresult[i] + lower_subresult[i]) / 2;
            }
            final_row.append(&mut upper_subresult);
        } else {
            final_row.append(&mut upper_subresult);
            final_lower_row.append(&mut lower_subresult);
        }
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
        new_vec.push((original_vec[2 * i] + original_vec[key]) / 2);
    }
    return new_vec;
}

#[cfg(test)]
mod tests {
    use super::{copy_and_pad, downsample_segment_of_row, downsample_vec_by_two};

    // TODO: test downsample_rows, downsample_channel

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
