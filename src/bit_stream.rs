use std::fs;

use crate::appendable_to_bit_stream::AppendableToBitStream;

/// Clear the last n bytes of the value.
/// TODO: find a better/not ugly way to do this.
///
/// # Arguments
///
/// * `value`: The value to work on.
/// * `n`: The amount of bytes to clear.
///
/// # Example
///
/// ```
/// let result = clear_last_n_bytes(7, 2);
/// assert_eq!(4, result);
/// ```
fn clear_last_n_bytes(value: u8, n: u8) -> u8 {
    let to_and = match n {
        0 => 0b11111111,
        1 => 0b11111110,
        2 => 0b11111100,
        3 => 0b11111000,
        4 => 0b11110000,
        5 => 0b11100000,
        6 => 0b11000000,
        7 => 0b10000000,
        _ => 0,
    };
    value & to_and
}

/// Clear the first n bytes of the value.
/// TODO: find a better/not ugly way to do this.
///
/// # Arguments
///
/// * `value`: The value to work on.
/// * `n`: The amount of bytes to clear.
///
/// # Example
///
/// ```
/// let result = clear_first_n_bytes(255, 2);
/// assert_eq!(63, result);
/// ```
fn clear_first_n_bytes(value: u8, n: u8) -> u8 {
    let to_and = match n {
        0 => 0b11111111,
        1 => 0b01111111,
        2 => 0b00111111,
        3 => 0b00011111,
        4 => 0b00001111,
        5 => 0b00000111,
        6 => 0b00000011,
        7 => 0b00000001,
        _ => 0,
    };
    value & to_and
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct BitStream {
    data: Vec<u8>,
    bits_in_last_byte: u8,
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
        if self.bits_in_last_byte == 8 {
            self.data.push(0);
            self.bits_in_last_byte = 0;
        }
        self.shift_and_add_to_last_byte(u8::from(value), 1);
    }

    /// Append a byte of data to this bit stream.
    /// integers of any size?
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

        let upper_value =
            clear_last_n_bytes(value, self.bits_in_last_byte) >> self.bits_in_last_byte;
        let bits_still_available_in_last_byte = 8 - self.bits_in_last_byte;
        self.shift_and_add_to_last_byte(upper_value, bits_still_available_in_last_byte);
        let lower_value = clear_first_n_bytes(value, bits_still_available_in_last_byte)
            << bits_still_available_in_last_byte;
        self.data.push(lower_value);
        self.bits_in_last_byte = previous_bits_in_last_byte;
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
        let mut index = 0;
        let mut last_byte = 0;
        if self.data.len() > 0 {
            index = self.data.len() - 1;
            last_byte = self.data[index];
        } else {
            self.data.push(last_byte);
        }
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
    pub fn flush_to_file(&self, filename: &str) -> std::io::Result<()> {
        fs::write(filename, &self.data)
    }

    pub fn append<T: AppendableToBitStream>(&mut self, value: T) {
        value.append(self);
    }
}


#[cfg(test)]
mod tests {
    use std::fs;

    use super::{BitStream, clear_first_n_bytes, clear_last_n_bytes};

    #[test]
    fn test_clear_first_n_bytes() {
        assert_eq!(0b11111111, clear_first_n_bytes(0b11111111, 0));
        assert_eq!(0b01111111, clear_first_n_bytes(0b11111111, 1));
        assert_eq!(0b00111111, clear_first_n_bytes(0b11111111, 2));
        assert_eq!(0b00011111, clear_first_n_bytes(0b11111111, 3));
        assert_eq!(0b00001111, clear_first_n_bytes(0b11111111, 4));
        assert_eq!(0b00000111, clear_first_n_bytes(0b11111111, 5));
        assert_eq!(0b00000011, clear_first_n_bytes(0b11111111, 6));
        assert_eq!(0b00000001, clear_first_n_bytes(0b11111111, 7));
        assert_eq!(0b00000000, clear_first_n_bytes(0b11111111, 8));
    }

    #[test]
    fn test_clear_last_n_bytes() {
        assert_eq!(0b11111111, clear_last_n_bytes(0b11111111, 0));
        assert_eq!(0b11111110, clear_last_n_bytes(0b11111111, 1));
        assert_eq!(0b11111100, clear_last_n_bytes(0b11111111, 2));
        assert_eq!(0b11111000, clear_last_n_bytes(0b11111111, 3));
        assert_eq!(0b11110000, clear_last_n_bytes(0b11111111, 4));
        assert_eq!(0b11100000, clear_last_n_bytes(0b11111111, 5));
        assert_eq!(0b11000000, clear_last_n_bytes(0b11111111, 6));
        assert_eq!(0b10000000, clear_last_n_bytes(0b11111111, 7));
        assert_eq!(0b00000000, clear_last_n_bytes(0b11111111, 8));
    }

    #[test]
    fn test_flush_to_file() -> std::io::Result<()> {
        let stream = BitStream {
            data: vec![0b10101010, 0b01010101],
            bits_in_last_byte: 0,
        };
        let filename = "test.bin";
        stream.flush_to_file(filename)?;

        let contents = fs::read(filename)?;
        assert_eq!(vec![0b10101010, 0b01010101], contents);

        // Clean up the file
        fs::remove_file(filename)?;

        Ok(())
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
        assert_eq!(vec![0b10100000], stream.data);
        assert_eq!(3, stream.bits_in_last_byte);
    }

    #[test]
    fn test_generic_append_bit_vec() {
        let mut stream = BitStream::open();
        let bits = vec![true, false, true];
        stream.append(bits);
        assert_eq!(vec![0b10100000], stream.data);
        assert_eq!(3, stream.bits_in_last_byte);
    }

    #[test]
    fn test_generic_append_byte_vec() {
        let mut stream = BitStream::open();
        let bytes = vec![127, 4, 255];
        stream.append(bytes);
        assert_eq!(vec![127, 4, 255], stream.data);
        assert_eq!(8, stream.bits_in_last_byte);
    }

    #[test]
    fn test_generic_append_bytes_only() {
        let mut stream = BitStream::open();
        stream.append(44);
        stream.append(231);
        assert_eq!(vec![44, 231], stream.data);
        assert_eq!(8, stream.bits_in_last_byte);
    }

    #[test]
    fn test_generic_append_bits_and_bytes() {
        let mut stream = BitStream::open();
        stream.append(44);
        stream.append(false);
        stream.append(true);
        stream.append(255);
        assert_eq!(vec![44, 0b01111111, 0b11000000], stream.data);
        assert_eq!(2, stream.bits_in_last_byte);
    }

    #[test]
    fn test_read_bit_stream_from_file() {
        let stream = BitStream {
            data: vec![1, 2, 3, 4, 5, 6, 7, 8],
            bits_in_last_byte: 0,
        };
        let filename = "test/binary_stream_test_file.bin";

        let bit_stream = BitStream::read_bit_stream_from_file(filename);
        assert_eq!(stream, bit_stream);
    }
}
