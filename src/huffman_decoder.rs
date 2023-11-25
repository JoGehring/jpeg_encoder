use std::collections::HashMap;

use crate::bit_stream::BitStream;

/// Decode a huffman encoded bit stream.
///
/// # Arguments
///
/// * `stream`: The stream of data to decode.
/// * `code`: The code to decode it with, which should be output by huffman::encode().
pub fn decode(stream: &mut BitStream, code: HashMap<u8, (u8, u16)>) -> BitStream {
    let mut result = BitStream::open();

    let (canonical_table, max_len) = create_canonical_table(code);

    // let last_code = canonical_table.last().unwrap().0;
    let mut iter = 1;
    while !stream.is_empty() {
        let value = stream.read_n_bits_padded(max_len, true);
        let mut symbol = 0;
        for (code, sym, len) in &canonical_table {
            if *code < value {
                continue;
            }
            symbol = *sym;
            stream.flush_n_bits(*len);
            break;
        }
        iter += 1;
        if iter >= 100 {
            assert!(false)
        }

        result.append(symbol);
    }

    result
}

/// Create the table needed for canonical search.
///
/// # Arguments
///
/// * `code`: The code to create a canonical table for.
fn create_canonical_table(code: HashMap<u8, (u8, u16)>) -> (Vec<(u16, u8, u8)>, u8) {
    let max_len = get_max_len_from_code(&code);

    let mut all_codes = create_code_vec_from_map(&code);

    sort_code_vector_by_length_and_code(&mut all_codes);

    all_codes = pad_code_with_ones(&mut all_codes, max_len);

    (all_codes, max_len)
}

/// Extract the maximum codeword length from the code.
///
/// # Arguments
///
/// * `code`: The code to work with.
fn get_max_len_from_code(code: &HashMap<u8, (u8, u16)>) -> u8 {
    code.iter()
        .max_by(|value1, value2| value1.1.cmp(value2.1))
        .unwrap()
        .1
        .0
}

/// Turn a HashMap mapping symbols to their code into a vector with both symbols and code.
/// The vector contains tuples of (code, symbol, code_length).
///
/// # Arguments
///
/// * `code`: The code to convert.
fn create_code_vec_from_map(code: &HashMap<u8, (u8, u16)>) -> Vec<(u16, u8, u8)> {
    code.into_iter()
        .map(|(symbol, len_and_code)| (len_and_code.1, *symbol, len_and_code.0))
        .collect()
}

/// Sort the code vector by length first, and by the code if length is equal.
///
/// # Arguments
///
/// * `all_codes`: The vector to sort.
fn sort_code_vector_by_length_and_code(all_codes: &mut Vec<(u16, u8, u8)>) {
    all_codes.sort_by(|a, b| {
        if a.2 == b.2 {
            return a.0.cmp(&b.0);
        }
        return a.2.cmp(&b.2);
    });
}

/// Pad each code in the given code vector with 1, until all codes are of the length max_len.
///
/// # Arguments
///
/// * `all_codes`: The vector to work on.
/// * `max_len`: The length of the longest code in `all_codes`.
fn pad_code_with_ones(all_codes: &mut Vec<(u16, u8, u8)>, max_len: u8) -> Vec<(u16, u8, u8)> {
    all_codes
        .iter()
        .map(|v| {
            let mut new_code = v.0;
            for _ in 0..(max_len - v.2) {
                new_code = new_code << 1;
                new_code += 1;
            }
            (new_code, v.1, v.2)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{bit_stream::BitStream, huffman::encode, huffman_decoder::{create_code_vec_from_map, get_max_len_from_code, sort_code_vector_by_length_and_code}};

    use super::{create_canonical_table, decode, pad_code_with_ones};

    #[test]
    fn test_encode_and_decode() {
        let mut plain_text = BitStream::open();
        plain_text.append_byte(1);
        plain_text.append_byte(2);
        plain_text.append_byte(255);
        plain_text.append_byte(2);
        plain_text.append_byte(1);
        plain_text.append_byte(2);
        plain_text.append_byte(2);
        plain_text.append_byte(1);
        plain_text.append_byte(255);

        let (mut encoded_text, map) = encode(&mut plain_text);
        let decoded_stream = decode(&mut encoded_text, map);
        assert_eq!(vec![1, 2, 255, 2, 1, 2, 2, 1, 255], *decoded_stream.data());
    }

    #[test]
    fn test_create_canonical_table() {
        let mut code = HashMap::new();
        code.insert(1, (2, 16));
        code.insert(2, (3, 32));
        code.insert(3, (4, 64));
        code.insert(4, (5, 128));

        let (canonical_table, max_len) = create_canonical_table(code);

        assert_eq!(canonical_table, vec![(135, 1, 2), (131, 2, 3), (129, 3, 4), (128, 4, 5)]);
        assert_eq!(max_len, 5);
    }

    #[test]
    fn test_get_max_len_from_code() {
        let mut code = HashMap::new();
        code.insert(1, (5, 100));
        code.insert(2, (7, 200));
        code.insert(3, (6, 300));

        assert_eq!(get_max_len_from_code(&code), 7);
    }

    #[test]
    fn test_create_code_vec_from_map() {
        let mut code = HashMap::new();
        code.insert(1, (5, 100));
        code.insert(2, (7, 200));
        code.insert(3, (6, 300));

        let mut result = create_code_vec_from_map(&code);
        result.sort();

        assert_eq!(result, vec![(100, 1, 5), (200, 2, 7), (300, 3, 6)]);
    }

    #[test]
    fn test_sort_code_vector_by_length_and_code() {
        let mut all_codes = vec![(1, 2, 3), (4, 5, 6), (7, 8, 2), (9, 10, 2)];
        sort_code_vector_by_length_and_code(&mut all_codes);

        assert_eq!(all_codes, vec![(7, 8, 2), (9, 10, 2), (1, 2, 3), (4, 5, 6)]);
    }

    #[test]
    fn test_pad_code_with_ones() {
        let mut all_codes = vec![(1, 2, 3), (4, 5, 6)];
        let max_len = 8;
        let result = pad_code_with_ones(&mut all_codes, max_len);

        assert_eq!(result, vec![(63, 2, 3), (19, 5, 6)]);
    }
}