use std::collections::HashMap;

const CATEGORY_OFFSET: [i32; 15] = [
    0b1,
    0b11,
    0b111,
    0b1111,
    0b1111_1,
    0b1111_11,
    0b1111_111,
    0b1111_1111,
    0b1111_1111_1,
    0b1111_1111_11,
    0b1111_1111_111,
    0b1111_1111_1111,
    0b1111_1111_1111_1,
    0b1111_1111_1111_11,
    0b1111_1111_1111_111,
];

/// Get the DC coefficients from the given values.
pub fn dc_coefficients(values: &Vec<[i32; 64]>) -> Vec<i32> {
    values.iter().map(|val| val[0]).collect()
}

/// Get the AC coefficients from the given values.
pub fn ac_coefficients(values: &Vec<[i32; 64]>) -> Vec<Vec<i32>> {
    values.iter().map(|val| (&val[1..64]).to_owned()).collect()
}

/// Encode a set of DC coefficients.
/// Coefficients are first replaced by the difference between
/// them and the previous coefficient, then categorized.
/// The categories are huffman encoded.
/// Returns both the now encoded values and the huffman code map.
pub fn encode_dc_coefficients(
    dc_coefficients: &Vec<i32>,
) -> (Vec<((u8, u16), u16)>, HashMap<u8, (u8, u16)>) {
    let diffs: Vec<i32> = coefficients_to_diffs(dc_coefficients);

    categorize_and_encode_diffs(&diffs)
}

/// Encode two sets of DC coefficients.
/// Coefficients are first replaced by the difference between
/// them and the previous coefficient,
/// then the sets are combined and categorized.
/// The categories are huffman encoded.
/// Returns both the now encoded values (first the ones from dc_coefficients_1, then dc_coefficients_2)
/// and the huffman code map.
pub fn encode_two_dc_coefficients(
    dc_coefficients_1: &Vec<i32>,
    dc_coefficients_2: &Vec<i32>,
) -> (Vec<((u8, u16), u16)>, HashMap<u8, (u8, u16)>) {
    let mut diffs: Vec<i32> = coefficients_to_diffs(dc_coefficients_1);
    diffs.append(&mut coefficients_to_diffs(dc_coefficients_2));

    categorize_and_encode_diffs(&diffs)
}

/// Get the differences between adjacent coefficients.
fn coefficients_to_diffs(coefficients: &Vec<i32>) -> Vec<i32> {
    let mut diffs: Vec<i32> = Vec::with_capacity(coefficients.len());
    let mut prev = 0;
    for coeff in coefficients {
        diffs.push(coeff - prev);
        prev = *coeff;
    }
    diffs
}

/// Categorize the given coefficient differences, then huffman
/// encode the categories and return the encoded differences as well as the
/// huffman code map.
fn categorize_and_encode_diffs(
    diffs: &Vec<i32>,
) -> (Vec<((u8, u16), u16)>, HashMap<u8, (u8, u16)>) {
    let categorized: Vec<(u8, u16)> = diffs.iter().map(|diff| categorize(*diff)).collect();

    let mut categories = crate::BitStream::open();
    categories.append(categorized.iter().map(|cat| cat.0).collect::<Vec<u8>>());
    let category_code = crate::huffman::parse_u8_stream(&mut categories).code_map();

    (
        categorized
            .iter()
            .map(|cat| (*category_code.get(&cat.0).unwrap(), cat.1))
            .collect(),
        category_code,
    )
}

/// Get the categorised representation of the given value.
/// Values get a category between 0 and 15 based on the amount
/// of bits set. For negative values, an offset is applied
/// (so the lowest value, e.g. -31 for category 5, translates to
/// 0* as a bit representation).
pub fn categorize(value: i32) -> (u8, u16) {
    if value == 0 {
        return (0, u16::MAX);
    }
    let cat = 32 - value.abs().leading_zeros() as u8;
    if value.signum() == -1 {
        let offset = CATEGORY_OFFSET[(cat - 1) as usize];
        (cat, (value + offset) as u16)
    } else {
        (cat, value as u16)
    }
}

#[cfg(test)]
mod tests {
    use super::{ac_coefficients, categorize, coefficients_to_diffs, dc_coefficients};

    #[test]
    fn test_get_dc_coefficients() {
        let values = vec![
            [2; 64],
            [1; 64],
            [
                1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 1, 2, 3, 4, 5, 6, 7, 8, 9,
                10, 11, 12, 13, 14, 15, 16, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16,
                1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16,
            ],
        ];
        let expected = vec![2, 1, 1];
        let actual = dc_coefficients(&values);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_get_ac_coefficients() {
        let values = vec![
            [2; 64],
            [1; 64],
            [
                1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 1, 2, 3, 4, 5, 6, 7, 8, 9,
                10, 11, 12, 13, 14, 15, 16, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16,
                1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16,
            ],
        ];
        let expected = vec![
            vec![2; 63],
            vec![1; 63],
            vec![
                2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10,
                11, 12, 13, 14, 15, 16, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 1,
                2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16,
            ],
        ];
        let actual = ac_coefficients(&values);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_coefficients_to_diffs() {
        let coeffs: Vec<i32> = vec![-120, 20, 100, -1, 90];
        let expected: Vec<i32> = vec![-120, 140, 80, -101, 91];
        let actual = coefficients_to_diffs(&coeffs);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_categorize() {
        let max_val = categorize(32767);
        assert_eq!((15, 0b1111_1111_1111_111), max_val);
        let min_val = categorize(-32767);
        assert_eq!((15, 0b0), min_val);
        let zero = categorize(0);
        assert_eq!((0, 0b1111_1111_1111_1111), zero);
        let minus_one = categorize(-1);
        assert_eq!((1, 0b0), minus_one);
        let one = categorize(1);
        assert_eq!((1, 0b1), one);
        let border_u8_neg = categorize(-255);
        assert_eq!((8, 0b0), border_u8_neg);
        let border_u8_pos = categorize(255);
        assert_eq!((8, 0b1111_1111), border_u8_pos);
        let border_8_neg = categorize(-128);
        assert_eq!((8, 0b0111_1111), border_8_neg);
        let border_8_pos = categorize(128);
        assert_eq!((8, 0b1000_0000), border_8_pos);
        let anywhere_neg = categorize(-3153);
        assert_eq!((12, 942), anywhere_neg);
        let anywhere_pos = categorize(3153);
        assert_eq!((12, 3153), anywhere_pos);
    }
}
