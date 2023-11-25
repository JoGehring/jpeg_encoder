use std::fs;

use crate::{appendable_to_bit_stream::AppendableToBitStream, utils::get_n_bits_at_offset};

#[derive(Clone, Debug, PartialEq)]
pub struct BitStream {
    data: Vec<u8>,
    bits_in_last_byte: u8,
    bits_read_from_first_byte: u8,
}

/// Pad the first passed-in `value´ with the given `pad`, so th
fn pad_read_bit_result(mut value: u16, amount: u8, pad: bool) -> u16 {
    let pad_u16 = pad as u16;
    for _ in 0..amount {
        value = (value << 1) + pad_u16;
    }
    value
}

impl BitStream {
    /// Open a bit stream.
    ///
    /// # Example
    ///
    /// ```
    /// let mut stream = BitStream::open();
    /// ```
    pub fn open() -> BitStream {
        BitStream {
            ..Default::default()
        }
    }

    /// Create a BitStream object from a file.
    ///
    /// # Arguments
    ///
    /// * filename: The name of the file to write to.
    ///
    /// # Example
    ///
    /// ```
    /// let stream = BitStream::read_bit_stream_from_file(filename);
    /// stream.append_bit(true);
    /// ```
    pub fn read_bit_stream_from_file(filename: &str) -> BitStream {
        let data = fs::read(filename).expect("failed to read file");
        BitStream {
            data,
            bits_in_last_byte: 0,
            bits_read_from_first_byte: 0,
        }
    }

    /// Append a bit of data to this bit stream.
    ///
    /// # Arguments
    ///
    /// * value: Whether to append a 1 or 0.
    ///
    /// # Example
    ///
    /// ```
    /// let mut stream = BitStream::open();
    /// stream.append_bit(true);
    /// ```
    pub fn append_bit(&mut self, value: bool) {
        if self.bits_in_last_byte == 8 || self.bits_in_last_byte == 0 {
            self.data.push(if value { 0b1000_0000 } else { 0 });
            self.bits_in_last_byte = 1;
            return;
        }
        self.shift_and_add_to_last_byte(u8::from(value), 1);
    }

    /// Append a byte of data to this bit stream.
    ///
    /// # Arguments
    ///
    /// * value: The data to append.
    ///
    /// # Example
    ///
    /// ```
    /// let mut stream = BitStream.open();
    /// stream.append_byte(244);
    /// ```
    ///
    /// # Explanation
    ///
    /// ## upper_value
    ///
    /// The bits we can append to the last byte in the stream. Cut off the amount of bits already
    ///  occupied in the last byte and move the remaining towards the LSB, then add them to the last byte
    ///
    /// ## lower_value
    ///
    /// The value we have to append as a new byte. Cut off the bits we already appended to the last byte
    /// and move the remaining towards the MSB, then append as a new byte.
    ///
    /// ## General
    ///
    /// * If we have a fully filled byte at the end, we can just push the next to data
    /// * bits_in_last_byte doesn't change as we add a whole byte to the stream. We do need to store and re-set it though,
    ///     as shift_and_add_to_last_byte changes the value of bits_in_last_byte.
    pub fn append_byte(&mut self, value: u8) {
        // if the last byte in the stream is full, we can just append this one
        if self.bits_in_last_byte == 8 || self.bits_in_last_byte == 0 {
            self.data.push(value);
            self.bits_in_last_byte = 8;
            return;
        }

        let previous_bits_in_last_byte = self.bits_in_last_byte;

        let upper_value = value >> self.bits_in_last_byte;
        let bits_still_available_in_last_byte = 8 - self.bits_in_last_byte;
        self.shift_and_add_to_last_byte(upper_value, bits_still_available_in_last_byte);
        let lower_value = value << bits_still_available_in_last_byte;
        self.data.push(lower_value);
        self.bits_in_last_byte = previous_bits_in_last_byte;
    }

    /// Append the given amount of bits in value to the bit stream, starting from the MSB
    ///
    /// # Arguments
    ///
    /// * `value`: The value the bits are taken from
    /// * `amount`: The amount of bits to add to the stream
    ///
    /// # Example
    ///
    /// ```
    /// let mut stream = BitStream::open();
    /// stream.append_n_bits(128u8, 3); // necessary because shift_and_add_to_last_byte assumes a byte exists
    /// assert_eq!(vec![0b1000_0000], stream.data);
    /// assert_eq!(3, stream.bits_in_last_byte);
    /// ```
    ///
    /// # Explanation
    /// Mask all bits of value except at the respective position, then check if true or false. Then
    /// append the bit to the stream
    ///
    /// # Panics
    ///
    /// * Not implemented for Vecs or bools because not sensible
    /// * Amount bigger than bits in value
    pub fn append_n_bits<T: AppendableToBitStream>(&mut self, value: T, amount: u8) {
        value.append_n_bits(self, amount);
    }

    /// Pad the last byte with the specified value
    ///
    /// # Arguments
    ///
    /// * `value`: Wether it should be padded with ones or zeros
    ///
    /// # Example
    ///
    /// ```
    /// let mut stream = BitStream::open();
    /// stream.append_bit(false);
    /// stream.append_bit(true);
    /// stream.pad_last_byte(true);
    /// assert_eq!(vec![0b0111_1111], stream.data);
    /// assert_eq!(8, stream.bits_in_last_byte);
    /// ```
    pub fn pad_last_byte(&mut self, value: bool) {
        let amount = 8 - self.bits_in_last_byte;
        for _ in 0..amount {
            self.append_bit(value);
        }
    }

    /// Shift the provided value to the correct position, then store it in the last byte.
    /// This should be used to write data to the stream.
    ///
    /// # Arguments
    ///
    /// * `value`: The data to append. Only the first `shift` bits of this should be set.
    /// * `bits_to_occupy`: The amount of bits to add to the last byte.
    ///
    /// # Example
    ///
    /// ```
    /// let mut stream = BitStream::open();
    /// stream.append_byte(0); // necessary because shift_and_add_to_last_byte assumes a byte exists
    /// stream.shift_and_add_to_last_byte(3, 2);
    /// assert_eq!(vec![3], stream.data);
    /// ```
    ///
    /// # Explanation
    /// We shift the value to the correct position given by the available space in the last byte, then add the
    /// resulting byte to the last one and replace it within the vector
    ///
    /// # Panics
    ///
    /// * If more than the last `bits_to_occupy` bits of `value` are set
    fn shift_and_add_to_last_byte(&mut self, mut value: u8, bits_to_occupy: u8) {
        let index = self.data.len() - 1;
        let mut last_byte = self.data[index];
        let bits_available = 8 - bits_to_occupy - self.bits_in_last_byte;
        value = value << bits_available;
        last_byte += value;
        self.data[index] = last_byte;
        self.bits_in_last_byte += bits_to_occupy;
    }

    /// Flush the bit stream to a file.
    ///
    /// # Arguments
    ///
    /// * filename: The name of the file to write to.
    ///
    /// # Example
    ///
    /// ```
    /// let mut stream = BitStream.open();
    /// stream.append_bit(true);
    /// stream.append_bit(false);
    /// stream.flush_to_file("test.bin");
    /// ```
    pub fn flush_to_file(&self, filename: &str) {
        // todo clear first bits of self.data if self.read_bits_in_first_byte is set
        fs::write(filename, &self.data).expect("Error when writing to file.")
    }

    /// Read up to 16 bits from the stream. If the stream has less than the requested
    /// amount of bits, pad it with ones or zeroes depending on `pad`.
    /// This does *NOT* alter the data contained in the stream. Calling this method repeatedly without
    /// altering the stream inbetween leads to the same result.
    ///
    /// # Arguments
    ///
    /// * `amount`: The amount of bits to read. Should never be more than 16.
    /// * `pad`: Whether to pad the value with 1 or 0 if the stream has less than the requested amount of bits.
    pub fn read_n_bits_padded(&self, amount: u8, pad: bool) -> u16 {
        if self.is_empty() {
            let result = if pad { u16::MAX } else { u16::MIN };
            return result >> (16 - amount);
        }

        let mut result;
        let mut bits_in_result: u8 = 0;
        let mut byte_index = 1;

        result = self.read_n_bits_first_byte(&mut bits_in_result);

        // if we already have more data than we need, remove unneeded data and return
        if bits_in_result > amount {
            return result >> (bits_in_result - amount);
        }

        // if we don't have further data, pad and return
        if self.data.len() <= byte_index {
            return pad_read_bit_result(result, amount - bits_in_result, pad);
        }

        if (amount - bits_in_result) >= 8 {
            result = self.read_n_bits_middle_byte(&mut bits_in_result, &mut byte_index, result);

            // if we don't have further data, pad and return
            if self.data.len() <= byte_index {
                return pad_read_bit_result(result, amount - bits_in_result, pad);
            }
        }

        if amount > bits_in_result {
            result = self.read_n_bits_end(&mut bits_in_result, byte_index, result, amount);
        }

        pad_read_bit_result(result, amount - bits_in_result, pad)
    }

    /// Submethod of read_n_bits_padded().
    ///
    /// Read bits from the first byte of the stream.
    ///
    /// # Arguments
    ///
    /// * `bits_in_result`: out-parameter for the amount of bits in the resulting u16.
    fn read_n_bits_first_byte(&self, bits_in_result: &mut u8) -> u16 {
        let bits_in_first_byte = if self.data.len() == 1
            && !(self.bits_in_last_byte == 8 || self.bits_in_last_byte == 0)
        {
            self.bits_in_last_byte
        } else {
            8
        };
        *bits_in_result = bits_in_first_byte - self.bits_read_from_first_byte;
        let result = get_n_bits_at_offset(
            self.data[0],
            bits_in_first_byte - self.bits_read_from_first_byte,
            self.bits_read_from_first_byte,
        ) as u16;
        result
    }

    /// Submethod of read_n_bits_padded().
    ///
    /// Read bits from the byte_index'th byte of the stream.
    ///
    /// # Arguments
    ///
    /// * `bits_in_result`: Out-parameter, incremented by the amount of bits added to the result.
    /// * `byte_index`: The index of the byte we are reading in the data vector, incremented by 1 afterwards.
    /// * `result`: The existing result that this method adds to.
    fn read_n_bits_middle_byte(
        &self,
        bits_in_result: &mut u8,
        byte_index: &mut usize,
        mut result: u16,
    ) -> u16 {
        // if this is our last bit and is incomplete, only append what we have
        if self.data.len() == *byte_index - 1
            && !(self.bits_in_last_byte == 8 || self.bits_in_last_byte == 0)
        {
            result = (result << self.bits_in_last_byte)
                + get_n_bits_at_offset(self.data[*byte_index], self.bits_in_last_byte, 0) as u16;
            *bits_in_result += self.bits_in_last_byte;
        } else {
            // otherwise, just append the byte
            *bits_in_result += 8;
            result = (result << 8) + self.data[*byte_index] as u16;
        }
        *byte_index += 1;
        result
    }

    /// Submethod of read_n_bits_padded().
    ///
    /// Read bits from the byte_index'th byte of the stream. This is expected to result in `result` containing
    /// `amount` set bits, except if the byte does not contain that many bytes (i.e. it is at the end of the stream and incomplete).
    ///
    /// # Arguments
    ///
    /// * `bits_in_result`: Out-parameter, incremented by the amount of bits added to the result.
    /// * `byte_index`: The index of the byte we are reading in the data vector.
    /// * `result`: The existing result that this method adds to.
    /// * `amount`: The amount of bits the result is supposed to eventually contain.
    fn read_n_bits_end(
        &self,
        bits_in_result: &mut u8,
        byte_index: usize,
        mut result: u16,
        amount: u8,
    ) -> u16 {
        let number_of_bits = if self.data.len() == byte_index - 1
            && !(self.bits_in_last_byte == 8 || self.bits_in_last_byte == 0)
        {
            std::cmp::min(self.bits_in_last_byte, amount - *bits_in_result)
        } else {
            amount - *bits_in_result
        };

        result = (result << number_of_bits)
            + get_n_bits_at_offset(self.data[byte_index], number_of_bits, 0) as u16;

        *bits_in_result += number_of_bits;
        result
    }

    /// Flush the given number of bits from this stream.
    ///
    /// # Arguments
    ///
    /// * `amount`: The amount of bits to remove from the stream.
    pub fn flush_n_bits(&mut self, mut amount: u8) {
        if self.bits_read_from_first_byte + amount <= 7 {
            self.bits_read_from_first_byte += amount;
            return;
        }
        self.data.remove(0);
        amount -= 8 - self.bits_read_from_first_byte;
        while amount >= 8 {
            amount -= 8;
            self.data.remove(0);
        }
        self.bits_read_from_first_byte = amount;
        if self.data.len() <= 1 {
            if self.data.len() == 0 || self.bits_in_last_byte <= self.bits_read_from_first_byte {
                // if we're empty, reset the stream
                self.data = vec![];
                self.bits_in_last_byte = 0;
                self.bits_read_from_first_byte = 0;
            }
        }
    }

    /// Check whether this stream is empty, i.e. it no longer contains any data or all the data in it
    /// has already been read.
    pub fn is_empty(&self) -> bool {
        return self.data.len() == 0
            || (self.data.len() == 1 && self.bits_in_last_byte == self.bits_read_from_first_byte);
    }

    /// Append the given data to this bit stream.
    /// This is a wrapper function for AppendableToBitStream::append().
    ///
    /// # Arguments
    ///
    /// * value: The data to append.
    ///
    /// # Example
    ///
    /// ```
    /// let mut stream = BitStream.open();
    /// stream.append(244u8);
    /// stream.append(244u16);
    /// ```
    pub fn append<T: AppendableToBitStream>(&mut self, value: T) {
        value.append(self);
    }

    pub fn data(&self) -> &Vec<u8> {
        &self.data
    }
    pub fn bits_in_last_byte(&self) -> u8 {
        self.bits_in_last_byte
    }
}

impl Default for BitStream {
    fn default() -> BitStream {
        BitStream {
            data: Vec::with_capacity(4096),
            bits_in_last_byte: 0,
            bits_read_from_first_byte: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use rand::Rng;

    use super::BitStream;

    #[test]
    fn test_is_empty_empty_stream() {
        let stream = BitStream::open();
        assert!(stream.is_empty());
    }

    #[test]
    fn test_is_empty_non_empty_stream() {
        let mut stream = BitStream::open();
        stream.append(244u8);
        assert!(!stream.is_empty());
    }

    #[test]
    fn test_append_u8() {
        let mut stream = BitStream::open();
        stream.append(244u8);
        assert_eq!(stream.data, vec![244]);
    }

    #[test]
    fn test_append_u16() {
        let mut stream = BitStream::open();
        stream.append(244u16);
        assert_eq!(stream.data, vec![0, 244]);
    }

    #[test]
    fn test_flush_n_bits_less_than_byte() {
        let mut stream = BitStream::open();
        stream.append(244u8);
        stream.flush_n_bits(3);
        assert_eq!(stream.data, vec![244]);
        assert_eq!(stream.bits_read_from_first_byte, 3);
    }

    #[test]
    fn test_flush_n_bits_multiple_bytes() {
        let mut stream = BitStream::open();
        stream.append(244u8);
        stream.append(255u8);
        stream.flush_n_bits(13);
        assert_eq!(stream.data, vec![255]);
        assert_eq!(stream.bits_read_from_first_byte, 5);
    }

    #[test]
    fn test_flush_n_bits_empty_stream() {
        let mut stream = BitStream::open();
        stream.flush_n_bits(5);
        assert_eq!(stream.data, vec![]);
        assert_eq!(stream.bits_read_from_first_byte, 5);
    }

    #[test]
    fn test_read_n_bits_padded_empty_stream_pad_true() {
        let stream = BitStream::open();
        let result = stream.read_n_bits_padded(8, true);
        assert_eq!(result, 0b1111_1111);
    }

    #[test]
    fn test_read_n_bits_padded_empty_stream_pad_false() {
        let stream = BitStream::open();
        let result = stream.read_n_bits_padded(8, false);
        assert_eq!(result, u16::MIN);
    }

    #[test]
    fn test_read_n_bits_padded_sufficient_data_no_padding() {
        let mut stream = BitStream::open();
        stream.append_byte(0b1100_0011);
        let result = stream.read_n_bits_padded(8, false);
        assert_eq!(result, 0b1100_0011);
    }

    #[test]
    fn test_read_n_bits_padded_sufficient_data_with_padding() {
        let mut stream = BitStream::open();
        stream.append_byte(0b1100_0011);
        let result = stream.read_n_bits_padded(10, true);
        assert_eq!(result, 0b0011_0000_1111);
    }

    #[test]
    fn test_flush_to_file() -> std::io::Result<()> {
        let stream = BitStream {
            data: vec![0b10101010, 0b01010101],
            bits_in_last_byte: 0,
            bits_read_from_first_byte: 0,
        };
        let filename = "test.bin";
        stream.flush_to_file(filename);

        let contents = fs::read(filename)?;
        assert_eq!(vec![0b10101010, 0b01010101], contents);

        // Clean up the file
        fs::remove_file(filename)
    }

    #[test]
    fn test_append_bits() {
        let mut stream = BitStream::open();
        stream.append_bit(true);
        stream.append_bit(false);
        stream.append_bit(true);
        stream.append_bit(true);
        assert_eq!(vec![0b10110000], stream.data);
        assert_eq!(4, stream.bits_in_last_byte);
    }

    #[test]
    fn test_append_bytes() {
        let mut stream = BitStream::open();
        stream.append_byte(44);
        stream.append_byte(231);
        assert_eq!(vec![44, 231], stream.data);
        assert_eq!(8, stream.bits_in_last_byte);
    }

    #[test]
    fn test_append_bits_and_bytes() {
        let mut stream = BitStream::open();
        stream.append_byte(44);
        stream.append_bit(false);
        stream.append_bit(true);
        stream.append_byte(255);
        assert_eq!(vec![44, 0b01111111, 0b11000000], stream.data);
        assert_eq!(2, stream.bits_in_last_byte);
    }

    #[test]
    fn test_generic_append_bits_only() {
        let mut stream = BitStream::open();
        stream.append(true);
        stream.append(false);
        stream.append(true);
        assert_eq!(vec![0b1010_0000], stream.data);
        assert_eq!(3, stream.bits_in_last_byte);
    }

    #[test]
    fn test_generic_append_bit_vec() {
        let mut stream = BitStream::open();
        let bits = vec![true, false, true];
        stream.append(bits);
        assert_eq!(vec![0b1010_0000], stream.data);
        assert_eq!(3, stream.bits_in_last_byte);
    }

    #[test]
    fn test_generic_append_byte_vec() {
        let mut stream = BitStream::open();
        let bytes: Vec<u8> = vec![127, 4, 255];
        stream.append(bytes);
        assert_eq!(vec![127, 4, 255], stream.data);
        assert_eq!(8, stream.bits_in_last_byte);
    }

    #[test]
    fn test_generic_append_bytes_only() {
        let mut stream = BitStream::open();
        stream.append::<u8>(44);
        stream.append::<u8>(231);
        assert_eq!(vec![44, 231], stream.data);
        assert_eq!(8, stream.bits_in_last_byte);
    }

    #[test]
    fn test_generic_append_first_bytes_then_bits() {
        let mut stream = BitStream::open();
        stream.append::<u8>(44);
        stream.append(false);
        stream.append(true);
        stream.append::<u8>(255);
        assert_eq!(vec![44, 0b0111_1111, 0b1100_0000], stream.data);
        assert_eq!(2, stream.bits_in_last_byte);
    }

    #[test]
    fn test_generic_append_first_bits_then_bytes() {
        let mut stream = BitStream::open();
        stream.append(true);
        stream.append(false);
        stream.append(true);
        stream.append::<u8>(255);
        stream.append(false);
        stream.append_byte(9);
        assert_eq!(vec![0b1011_1111, 0b1110_0000, 0b1001_0000], stream.data);
        assert_eq!(4, stream.bits_in_last_byte);
    }

    #[test]
    fn test_read_bit_stream_from_file() {
        let stream = BitStream {
            data: vec![1, 2, 3, 4, 5, 6, 7, 8],
            bits_in_last_byte: 0,
            bits_read_from_first_byte: 0,
        };
        let filename = "test/binary_stream_test_file.bin";

        let bit_stream = BitStream::read_bit_stream_from_file(filename);
        assert_eq!(stream, bit_stream);
    }

    #[test]
    fn test_generic_append_u16_vec() {
        let mut stream = BitStream::open();
        let bytes: Vec<u16> = vec![0x1412, 0xffff, 0xfafe];
        stream.append(bytes);
        assert_eq!(vec![0x14, 0x12, 0xff, 0xff, 0xfa, 0xfe], stream.data);
        assert_eq!(8, stream.bits_in_last_byte);
    }

    #[test]
    fn test_generic_append_u16_only() {
        let mut stream = BitStream::open();
        stream.append::<u16>(0x1234);
        stream.append::<u16>(0xfef0);
        assert_eq!(vec![0x12, 0x34, 0xfe, 0xf0], stream.data);
        assert_eq!(8, stream.bits_in_last_byte);
    }

    #[test]
    fn test_append_n_bits_u8() {
        let mut stream = BitStream::open();
        stream.append_n_bits(0b1011_1111u8, 7);
        assert_eq!(vec![0b0111_1110], stream.data);
        assert_eq!(7, stream.bits_in_last_byte);
    }

    #[test]
    #[should_panic]
    fn test_append_n_bits_u8_amount_to_big() {
        let mut stream = BitStream::open();
        stream.append_n_bits(0b1011_1111u8, 12);
    }

    #[test]
    fn test_append_n_bits_u16() {
        let mut stream = BitStream::open();
        stream.append_n_bits::<u16>(0b1011_1100_1010_1011, 13);
        assert_eq!(vec![0b1110_0101, 0b0101_1000], stream.data);
        assert_eq!(5, stream.bits_in_last_byte);
    }

    #[test]
    #[should_panic]
    fn test_append_n_bits_u16_amount_to_big() {
        let mut stream = BitStream::open();
        stream.append_n_bits::<u16>(0b1011_0000_0000_0000, 29);
    }

    #[test]
    fn test_append_n_bits_vec8() {
        let mut stream = BitStream::open();
        stream.append_n_bits::<Vec<u8>>(vec![0b1010_1010, 0b1010_1010, 0b1010_1010], 19);
        assert_eq!(vec![0b1010_1010, 0b1010_1010, 0b1010_0000], stream.data);
        assert_eq!(3, stream.bits_in_last_byte);
    }

    #[test]
    fn test_pad_last_byte_true() {
        let mut stream = BitStream::open();
        stream.append_bit(true);
        stream.append_bit(false);
        stream.append_bit(true);
        stream.pad_last_byte(true);

        assert_eq!(vec![0b101_1_1111], stream.data);
        assert_eq!(8, stream.bits_in_last_byte);
    }

    #[test]
    fn test_pad_last_byte_false() {
        let mut stream = BitStream::open();
        stream.append_bit(true);
        stream.append_bit(false);
        stream.append_bit(true);
        stream.pad_last_byte(false);

        assert_eq!(vec![0b101_0_0000], stream.data);
        assert_eq!(8, stream.bits_in_last_byte);
    }

    #[test]
    #[should_panic]
    fn test_append_n_bits_vec8_amount_to_big() {
        let mut stream = BitStream::open();
        stream.append_n_bits::<Vec<u8>>(vec![0b1010_1010, 0b1010_1010, 0b1010_1010], 59);
    }

    #[test]
    fn test_append_n_bits_vec16() {
        let mut stream = BitStream::open();
        stream.append_n_bits::<Vec<u16>>(
            vec![
                0b1010_1010_1010_1010,
                0b1010_1010_1010_1010,
                0b1010_1010_1010_1010,
            ],
            35,
        );
        assert_eq!(
            vec![
                0b1010_1010,
                0b1010_1010,
                0b1010_1010,
                0b1010_1010,
                0b1010_0000,
            ],
            stream.data
        );
        assert_eq!(3, stream.bits_in_last_byte);
    }

    #[test]
    #[should_panic]
    fn test_append_n_bits_vec16_amount_to_big() {
        let mut stream = BitStream::open();
        stream.append_n_bits::<Vec<u16>>(vec![0b1010_1010, 0b1010_1010, 0b1010_1010], 59);
    }

    #[test]
    #[ignore]
    fn test_append_large_random_data() {
        // TODO: simplify, as rng automatically generates random vecs
        let mut stream = BitStream::open();
        let mut rng = rand::thread_rng();
        let tested_capacity: u64 = 10_000_000_000;
        let mut bit_vec: Vec<bool> = Vec::with_capacity(tested_capacity as usize);
        for _ in 0..tested_capacity {
            let n1: bool = rng.gen();
            bit_vec.push(n1);
            stream.append(n1);
        }
        for i in 0..tested_capacity {
            let expected_val = bit_vec[i as usize];
            let actual_val = (stream.data()[(i / 8) as usize] & (0b1000_0000 >> i % 8)) != 0;
            assert_eq!(expected_val, actual_val);
        }
    }
}
