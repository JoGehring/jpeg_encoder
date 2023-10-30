use std::fs;

use criterion::{black_box, Criterion, criterion_group, criterion_main};

// Due to limitations with Criterion, we need to copy/paste bit_stream.rs here.
// We can only use code from src/ if we are creating a library :/

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
            stream.append(*val);
        }
    }
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

        let upper_value = value >> self.bits_in_last_byte;
        let bits_still_available_in_last_byte = 8 - self.bits_in_last_byte;
        self.shift_and_add_to_last_byte(upper_value, bits_still_available_in_last_byte);
        let lower_value = value << bits_still_available_in_last_byte;
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
    pub fn data(&self) -> &Vec<u8> {
        &self.data
    }
    pub fn bits_in_last_byte(&self) -> u8 {
        self.bits_in_last_byte
    }
}

pub fn criterion_bit_benchmark(c: &mut Criterion) {
    c.bench_function("Test append_bit", |b| {
        b.iter(|| {
            let mut stream = BitStream::open();
            for _ in 0..10000000 {
                stream.append_bit(black_box(true));
            }
        })
    });
}

pub fn criterion_byte_benchmark(c: &mut Criterion) {
    c.bench_function("Test append_byte", |b| {
        b.iter(|| {
            let mut stream = BitStream::open();
            stream.append_bit(black_box(true));
            stream.append_bit(black_box(true));
            for _ in 0..10000000 {
                stream.append_byte(black_box(170));
            }
        })
    });
}

pub fn criterion_byte_and_write_benchmark(c: &mut Criterion) {
    c.bench_function("Test append_byte and flush to file", |b| {
        b.iter(|| {
            let mut stream = BitStream::open();
            stream.append_bit(black_box(true));
            stream.append_bit(black_box(true));
            for _ in 0..10000000 {
                stream.append_byte(black_box(170));
            }
            stream.flush_to_file(black_box("test/test.bin")).expect("bit stream could not be flushed to file");
        })
    });
    fs::remove_file("test/test.bin").expect("file could not be removed");
}

pub fn criterion_read_benchmark(c: &mut Criterion) {
    let mut stream = BitStream::open();
    for _ in 0..10000000 {
        stream.append_byte(170);
    }
    stream.flush_to_file(black_box("test/test.bin")).expect("bit stream could not be flushed to file");
    c.bench_function("Test reading bitstream from file", |b| {
        b.iter(|| {
            let mut read_stream = BitStream::read_bit_stream_from_file(black_box("test/test.bin"));
            read_stream.append(black_box(true));
        })
    });
    fs::remove_file("test/test.bin").expect("file could not be removed");
}

pub fn criterion_read_and_write_benchmark(c: &mut Criterion) {
    let mut stream = BitStream::open();
    for _ in 0..10000000 {
        stream.append_byte(170);
    }
    stream.flush_to_file(black_box("test/test.bin")).expect("bit stream could not be flushed to file");
    c.bench_function("Test reading and writing bitstream from/to file", |b| {
        b.iter(|| {
            let mut read_stream = BitStream::read_bit_stream_from_file(black_box("test/test.bin"));
            for _ in 0..10000000 {
                read_stream.append_bit(false);
                read_stream.append_byte(black_box(170));
            }
            stream.flush_to_file(black_box("test/test.bin")).expect("bit stream could not be flushed to file");
        })
    });
    fs::remove_file("test/test.bin").expect("file could not be removed");
}

criterion_group!(benches, criterion_bit_benchmark, criterion_byte_benchmark, criterion_byte_and_write_benchmark, criterion_read_benchmark, criterion_read_and_write_benchmark);
criterion_main!(benches);