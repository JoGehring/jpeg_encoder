use crate::huffman::{HuffmanCode, HuffmanCodeMap};

/// a category code, containing the code length and code.
pub type CategoryCode = (u8, u16);

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
pub fn ac_coefficients(values: &Vec<[i32; 64]>) -> Vec<[i32; 63]> {
    values
        .iter()
        .map(|val| {
            let mut res = [1; 63];
            res[0..63].clone_from_slice(&val[1..64]);
            res
        })
        .collect()
}

/// Encode a set of DC coefficients.
/// Coefficients are first replaced by the difference between
/// them and the previous coefficient, then categorized.
/// The categories are huffman encoded.
/// Returns both the now encoded values and the huffman code map.
pub fn encode_dc_coefficients(
    dc_coefficients: &Vec<i32>,
) -> (Vec<(HuffmanCode, CategoryCode)>, HuffmanCodeMap) {
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
) -> (Vec<(HuffmanCode, CategoryCode)>, HuffmanCodeMap) {
    let mut diffs: Vec<i32> = coefficients_to_diffs(dc_coefficients_1);
    diffs.append(&mut coefficients_to_diffs(dc_coefficients_2));

    categorize_and_encode_diffs(&diffs)
}

/// Encode a set of AC coefficients.
/// Coefficients are first replaced by the zero runlength encoding and categorization,
/// then huffman encoded.
/// Returns both the now encoded values and the resulting huffman code map.
pub fn encode_ac_coefficients(
    ac_coefficients: &Vec<[i32; 63]>,
) -> (Vec<Vec<(HuffmanCode, CategoryCode)>>, HuffmanCodeMap) {
    let runlength_encoded: Vec<Vec<(u8, CategoryCode)>> = ac_coefficients
        .iter()
        .map(|coeff| runlength_encode_single_ac_table(coeff))
        .collect();
    huffman_encode_ac_coefficients(&runlength_encoded)
}

/// Encode two sets of AC coefficients.
/// Coefficients are first replaced by the zero runlength encoding and categorization,
/// then the sets are combined and huffman encoded.
/// Returns both the now encoded values (first the ones from ac_coefficients_1, then ac_coefficients_2)
/// and the resulting huffman code map.
pub fn encode_two_ac_coefficients(
    ac_coefficients_1: &Vec<[i32; 63]>,
    ac_coefficients_2: &Vec<[i32; 63]>,
) -> (Vec<Vec<(HuffmanCode, CategoryCode)>>, HuffmanCodeMap) {
    let mut runlength_encoded_1: Vec<Vec<(u8, CategoryCode)>> = ac_coefficients_1
        .iter()
        .map(|coeff| runlength_encode_single_ac_table(coeff))
        .collect();
    let mut runlength_encoded_2: Vec<Vec<(u8, CategoryCode)>> = ac_coefficients_2
        .iter()
        .map(|coeff| runlength_encode_single_ac_table(coeff))
        .collect();
    runlength_encoded_1.append(&mut runlength_encoded_2);
    huffman_encode_ac_coefficients(&runlength_encoded_1)
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
) -> (Vec<(HuffmanCode, CategoryCode)>, HuffmanCodeMap) {
    let categorized: Vec<CategoryCode> = diffs.iter().map(|diff| categorize(*diff)).collect();

    let mut categories = crate::BitStream::open();
    categories.append(categorized.iter().map(|cat| cat.0).collect::<Vec<u8>>());
    let category_code = crate::huffman::parse_u8_stream(&mut categories).code_map();

    (
        categorized
            .iter()
            .map(|cat| (*category_code.get(&cat.0).unwrap(), *cat))
            .collect(),
        category_code,
    )
}

///Run-length encode AC coefficients.
fn runlength_encode_single_ac_table(table: &[i32]) -> Vec<(u8, CategoryCode)> {
    let mut new_table: Vec<(u8, CategoryCode)> = Vec::with_capacity(63);
    let mut counter: u8 = 0;
    for (index, coefficient) in table.iter().enumerate() {
        if *coefficient != 0 {
            let (cat, code) = categorize(*coefficient);
            for _ in 0..counter / 16 {
                new_table.push((0xF0, (0, 0)));
            }
            // combined value of the amount of zeroes (upper 4 bytes) and the category (lower 4 bytes)
            let zeros_cat = ((counter % 16) << 4) + cat;
            new_table.push((zeros_cat, (cat, code)));
            counter = 0;
        } else if index == 62 {
            new_table.push((0, (0, 0)));
        } else {
            counter += 1;
        }
    }
    new_table
}

/// Create BitStream with all the chunk's categories, then huffman
/// encode the categories and return the resulting chunks with the zeros/category replaced with
/// huffman code as well as the huffman code map.
fn huffman_encode_ac_coefficients(
    runlength_encoded: &Vec<Vec<(u8, CategoryCode)>>,
) -> (Vec<Vec<(HuffmanCode, CategoryCode)>>, HuffmanCodeMap) {
    let mut categories = crate::BitStream::open();
    runlength_encoded
        .iter()
        .for_each(|table| table.iter().for_each(|val| categories.append(val.0)));

    let category_code = crate::huffman::parse_u8_stream(&mut categories).code_map();

    let mut huffman_encoded: Vec<Vec<(HuffmanCode, CategoryCode)>> =
        Vec::with_capacity(runlength_encoded.len());
    for table in runlength_encoded {
        let new_table: Vec<(HuffmanCode, CategoryCode)> = table
            .iter()
            .map(|cat| (*category_code.get(&cat.0).unwrap(), cat.1))
            .collect();
        huffman_encoded.push(new_table);
    }
    (huffman_encoded, category_code)
}

/// Get the categorised representation of the given value.
/// Values get a category between 0 and 15 based on the amount
/// of bits set. For negative values, an offset is applied
/// (so the lowest value, e.g. -31 for category 5, translates to
/// 0* as a bit representation).
pub fn categorize(value: i32) -> CategoryCode {
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

/// Re-order the Y coefficients to match the order they will be processed and printed out in.
/// Y coefficients are printed out in 2x2 blocks, going from left to right, then top to
/// bottom.
/// To achieve this, the coefficients are sorted by four criteria:
/// 1. The 2-row block they're in (so first rows 0-1, then rows 2-3, then 4-5, ...)
/// 2. The 2-column block they're in (so the first two values per row first, then the next two, ...)
/// From this sorting alone, coefficients are already sorted by the 2x2 blocks - the first four values are
/// the first block, the four values after that the second block, ...
/// So the second set of criteria is sorting within this block:
/// 3. The row the coefficients are in
/// 4. The column the coefficients are in
/// After the sorting, the values used for those sort criteria are taken out again.
pub fn reorder_y_coefficients<T: Copy>(coefficients: &mut Vec<T>, width: u16) {
    let blocks_per_row = width as usize / 8;
    let blocks_per_two_rows = blocks_per_row * 2;
    let mut vec = coefficients
        .iter()
        .enumerate()
        .map(|(idx, value)| {
            (
                value,
                idx / blocks_per_two_rows,  // first sort: 2-row-blocks
                (idx % blocks_per_row) / 2, // second sort: 2-column-blocks. Now we have each 2x2 block and just need to order within the blocks
                idx / blocks_per_row,       // third sort: rows
                idx,                        // fourth sort: columns
            )
        })
        .collect::<Vec<(&T, usize, usize, usize, usize)>>();

    vec.sort_by(|a, b| {
        // order by the sorting criteria from above
        if a.1 != b.1 {
            a.1.cmp(&b.1)
        } else if a.2 != b.2 {
            a.2.cmp(&b.2)
        } else if a.3 != b.3 {
            a.3.cmp(&b.3)
        } else {
            a.4.cmp(&b.4)
        }
    });

    // kinda inefficient copy here, but will work for now
    *coefficients = vec.iter().map(|input| *input.0).collect();
}

#[cfg(test)]
mod tests {
    use crate::coefficient_encoder::runlength_encode_single_ac_table;

    use super::{ac_coefficients, categorize, coefficients_to_diffs, dc_coefficients, reorder_y_coefficients};

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

    #[test]
    fn test_runlength_encode_slides() {
        let coefficients = vec![
            57, 45, 0, 0, 0, 0, 23, 0, -30, -16, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0,
        ];
        assert_eq!(63, coefficients.len());
        let expected = vec![
            (0x06, (6, 57)),
            (0x06, (6, 45)),
            (0x45, (5, 23)),
            (0x15, (5, 1)),
            (0x05, (5, 15)),
            (0x21, (1, 1)),
            (0, (0, 0)),
        ];
        let runlength_encoded = runlength_encode_single_ac_table(&coefficients);
        assert_eq!(expected, runlength_encoded);
    }

    #[test]
    fn test_runlength_encode_slides_many_zeros() {
        let coefficients = vec![
            57, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 2, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            895, 0, 0, 0, 0,
        ];
        assert_eq!(63, coefficients.len());
        let expected = vec![
            (0x06, (6, 57)),
            (0xF0, (0, 0)),
            (0x22, (2, 3)),
            (0x42, (2, 2)),
            (0xF0, (0, 0)),
            (0xF0, (0, 0)),
            (0x1A, (10, 895)),
            (0, (0, 0)),
        ];
        let runlength_encoded = runlength_encode_single_ac_table(&coefficients);
        assert_eq!(expected, runlength_encoded);
    }

    #[test]
    fn test_reorder_y_coefficients() {
        let width = 32;
        let mut actual = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
        reorder_y_coefficients(&mut actual, width);
        let expected = vec![1, 2, 5, 6, 3, 4, 7, 8, 9, 10, 13, 14, 11, 12, 15, 16];
        assert_eq!(expected, actual);
    }
}
