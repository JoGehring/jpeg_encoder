use crate::bit_stream::BitStream;

pub trait AppendableToBitStream {
    fn append(&self, stream: &mut BitStream);

    fn append_n_bits(&self, stream: &mut BitStream, amount: u8) { panic!("Not implemented for this type!") }
}

impl AppendableToBitStream for bool {
    fn append(&self, stream: &mut BitStream) {
        stream.append_bit(*self);
    }
}

impl AppendableToBitStream for Vec<bool> {
    fn append(&self, stream: &mut BitStream) {
        for val in self {
            stream.append_bit(*val);
        }
    }
}

impl AppendableToBitStream for u8 {
    fn append(&self, stream: &mut BitStream) {
        stream.append_byte(*self);
    }
    fn append_n_bits(&self, stream: &mut BitStream, amount: u8) {
        if amount > 8 { panic!("Not enough bits in value to append") }
        for pos in 0..amount {
            let i = 0b1000_0000 >> pos;
            let bit = self & i != 0;
            stream.append_bit(bit);
        }
    }
}

impl AppendableToBitStream for Vec<u8> {
    fn append(&self, stream: &mut BitStream) {
        for val in self {
            stream.append_byte(*val);
        }
    }

    fn append_n_bits(&self, stream: &mut BitStream, amount: u8) {
        if amount > (self.len() * 8) as u8 { panic!("Not enough bits in value to append") }
        for i in 0..amount {
            let current_val = self[(i / 8) as usize];
            let i = 0b1000_0000 >> i % 8;
            let bit = current_val & i != 0;
            stream.append_bit(bit);
        }
    }
}

impl AppendableToBitStream for u16 {
    fn append(&self, stream: &mut BitStream) {
        let bytes = self.to_be_bytes();
        stream.append_byte(bytes[0]);
        stream.append_byte(bytes[1]);
    }

    fn append_n_bits(&self, stream: &mut BitStream, amount: u8) {
        if amount > 16 { panic!("Not enough bits in value to append") }
        for pos in 0..amount {
            let i = 0b1000_0000_0000_0000 >> pos;
            let bit = self & i != 0;
            stream.append_bit(bit);
        }
    }
}

impl AppendableToBitStream for Vec<u16> {
    fn append(&self, stream: &mut BitStream) {
        for val in self {
            stream.append(*val);
        }
    }

    fn append_n_bits(&self, stream: &mut BitStream, amount: u8) {
        if amount > (self.len() * 16) as u8 { panic!("Not enough bits in value to append") }
        for i in 0..amount {
            let current_val = self[(i / 16) as usize];
            let i = 0b1000_0000_0000_0000 >> i % 16;
            let bit = current_val & i != 0;
            stream.append_bit(bit);
        }
    }
}
