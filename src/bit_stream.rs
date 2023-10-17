use std::fs;

#[derive(Clone, Debug, PartialEq)]
struct BitStream {
    data: Vec<u8>,
    bits_in_last_byte: u8,
}

impl BitStream {
    /// Open a bit stream.
    ///
    /// # Example
    ///
    /// ```
    /// let stream = BitStream::open();
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
    /// let stream = BitStream::open();
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
    /// let stream = BitStream.open();
    /// stream.append_byte(244);
    /// ```
    pub fn append_byte(&mut self, value: u8) {
        // if the last byte in the stream is full, we can just append this one
        if self.bits_in_last_byte == 8 {
            self.data.push(value);
            return;
        }
        panic!("Not implemented yet!");
        /*
        let upper_value = value >> self.bits_in_last_byte;
        let previous_bits_in_last_byte = self.bits_in_last_byte;
        self.shift_and_add_to_last_byte(upper_value, 8 - self.bits_in_last_byte);
        let lower_value = value << (8 - self.bits_in_last_byte);
        self.data.push(lower_value);
        self.bits_in_last_byte = previous_bits_in_last_byte;
        */
    }

    /// Shift the last byte and then add the provided value to it.
    /// This should be used to write data to the stream.
    /// TODO: proper doc comment
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
    /// let stream = BitStream.open();
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

#[cfg(test)]
mod tests {
    use std::fs;

    use super::BitStream;

    #[test]
    fn test_flush_to_file() {
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
    }

    #[test]
    fn test_append_bits() {
        let mut stream = BitStream::open();
        stream.append_bit(true);
        stream.append_bit(false);
        stream.append_bit(true);
        stream.append_bit(true);
        assert_eq!(11, stream.data[0]);
    }

    #[test]
    fn test_append_bytes() {
        let mut stream = BitStream::open();
        stream.append_byte(44);
        stream.append_byte(231);
        assert_eq!(vec![44, 231], stream.data);
    }

    #[test]
    fn test_append_bits_and_bytes() {
        let mut stream = BitStream::open();
        stream.append_byte(44);
        stream.append_bit(false);
        stream.append_bit(true);
        stream.append_byte(255);
        assert_eq!(vec![44, 127, 3], stream.data)
    }
}