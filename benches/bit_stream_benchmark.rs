use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::fs;

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

#[derive(Clone, Debug, PartialEq)]
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
    /// TODO: Perhaps also add a generic function to append
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
    pub fn append_byte(&mut self, value: u8) {
        // if the last byte in the stream is full, we can just append this one
        if self.bits_in_last_byte == 8 {
            self.data.push(value);
            return;
        }
        let upper_value =
            clear_last_n_bytes(value, self.bits_in_last_byte) >> self.bits_in_last_byte;
        let previous_bits_in_last_byte = self.bits_in_last_byte;
        self.shift_and_add_to_last_byte(upper_value, 8 - self.bits_in_last_byte);

        let lower_value = clear_first_n_bytes(value, 8 - previous_bits_in_last_byte);
        self.data.push(lower_value);
        self.bits_in_last_byte = previous_bits_in_last_byte;
    }

    /// Shift the last byte and then add the provided value to it.
    /// This should be used to write data to the stream.
    ///
    /// # Arguments
    ///
    /// * `value`: The data to append. Only the first `shift` bits of this should be set.
    /// * `shift`: The amount of bits to add to the last byte.
    ///
    /// # Example
    ///
    /// ```
    /// let mut stream = BitStream::open();
    /// stream.append_byte(0); // necessary because shift_and_add_to_last_byte assumes a byte exists
    /// stream.shift_and_add_to_last_byte(3, 2);
    /// assert_eq!(vec![3], stream.data);
    /// ```
    fn shift_and_add_to_last_byte(&mut self, value: u8, shift: u8) {
        let mut last_byte = self.data[self.data.len() - 1];
        last_byte = last_byte << shift;
        last_byte += value;
        let index = self.data.len() - 1;
        self.data[index] = last_byte;
        self.bits_in_last_byte += shift;
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
}

impl Default for BitStream {
    fn default() -> BitStream {
        BitStream {
            data: vec![],
            bits_in_last_byte: 8,
        }
    }
}

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("Test", |b| {
        b.iter(|| {
            let mut stream = BitStream::open();
            for i in 0..10000000 {
                stream.append_bit(i % 2 == 1);
            }
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);