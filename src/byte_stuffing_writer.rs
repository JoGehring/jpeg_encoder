use crate::bit_stream::BitStream;

pub struct ByteStuffingWriter {
    trailing_ones: u32,
}

impl ByteStuffingWriter {
    pub fn new() -> Self {
        Self { trailing_ones: 0 }
    }

    pub fn write_n_bits_to_stream(&mut self, stream: &mut BitStream, value: u16, mut amount: u8) {
        // do not append anything for amount 0
        if amount == 0 {
            return;
        }
        if amount > 8 {
            self.write_n_bits_to_stream_internal(stream, (value >> 8) as u8, amount - 8);
            amount = 8;
        }
        self.write_n_bits_to_stream_internal(stream, value as u8, amount);
    }

    fn write_n_bits_to_stream_internal(
        &mut self,
        stream: &mut BitStream,
        value: u8,
        mut amount: u8,
    ) {
        let value_left = value << (8 - amount);
        let total_ones = std::cmp::min(value_left.leading_ones(), amount as u32) + self.trailing_ones;
        // if we're at less than 8 total ones in sequence, simply write and update our struct, then we're done
        if total_ones < 8 {
            stream.append_n_bits(value, amount);
            let trailing_ones = std::cmp::min(value.trailing_ones(), amount as u32);
            if trailing_ones == amount as u32 {
                // for a 1* value, add the trailing ones to the old value rather than replace it
                self.trailing_ones += trailing_ones;
            } else {
                self.trailing_ones = trailing_ones;
            }
            return;
        }
        // otherwise we have to stuff a 0x00
        // so insert the 1s needed to fill up the 0xFF
        let remaining_ones = 8 - self.trailing_ones;
        stream.append_n_bits(0xFFFFu16, remaining_ones as u8);
        amount -= remaining_ones as u8;
        // then insert the 0x00
        stream.append::<u8>(0);
        // then insert the remaining data
        stream.append_n_bits(value, amount);
        let trailing_ones = std::cmp::min(value.trailing_ones(), amount as u32);
        self.trailing_ones = trailing_ones;
    }
}

impl Default for ByteStuffingWriter {
    fn default() -> Self {
        Self { trailing_ones: 0 }
    }
}

#[cfg(test)]
mod tests {
    use crate::bit_stream::BitStream;

    use super::ByteStuffingWriter;

    #[test]
    fn test_write_u16_ones_to_stream() {
        let mut stream = BitStream::open();
        let mut writer = ByteStuffingWriter::new();
        writer.write_n_bits_to_stream(&mut stream, 0xFFFF, 5);
        writer.write_n_bits_to_stream(&mut stream, 0xFFFF, 7);
        writer.write_n_bits_to_stream(&mut stream, 0xFFFF, 12);

        let expected: Vec<u8> = vec![0xFF, 0, 0xFF, 0, 0xFF, 0];
        assert_eq!(&expected, stream.data());
    }

    #[test]
    fn test_write_u16_alternating_to_stream() {
        let mut stream = BitStream::open();
        let mut writer = ByteStuffingWriter::new();
        writer.write_n_bits_to_stream(&mut stream, 0x00F0, 8);
        writer.write_n_bits_to_stream(&mut stream, 0x000F, 8);
        writer.write_n_bits_to_stream(&mut stream, 0x000F, 8);
        writer.write_n_bits_to_stream(&mut stream, 0x00F0, 8);
        writer.write_n_bits_to_stream(&mut stream, 0x0000, 16);

        let expected: Vec<u8> = vec![0xF0, 0x0F, 0x0F, 0xF0, 0x00, 0x00, 0x00];
        assert_eq!(&expected, stream.data());
    }
}