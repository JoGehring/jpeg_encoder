use crate::bit_stream::BitStream;

pub trait AppendableToBitStream {
    fn append(&self, stream: &mut BitStream);
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
}

impl AppendableToBitStream for Vec<u8> {
    fn append(&self, stream: &mut BitStream) {
        for val in self {
            stream.append_byte(*val);
        }
    }
}

impl AppendableToBitStream for u16 {
    fn append(&self, stream: &mut BitStream) {
        let bytes = self.to_be_bytes();
        stream.append_byte(bytes[0]);
        stream.append_byte(bytes[1]);
    }
}

impl AppendableToBitStream for Vec<u16> {
    fn append(&self, stream: &mut BitStream) {
        for val in self {
            let bytes = val.to_be_bytes();
            stream.append_byte(bytes[0]);
            stream.append_byte(bytes[1]);
        }
    }
}
