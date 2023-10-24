use crate::bit_stream::BitStream;

mod image;
mod downsample;
mod ppm_parser;
mod bit_stream;

fn main() {
    let mut stream = BitStream::open();
    stream.append_bit(true);
    stream.append_bit(true);
    stream.append_bit(true);
    stream.append_bit(true);
    stream.append_bit(true);
    stream.append_bit(true);
    stream.append_bit(true);

    stream.append_byte(3);
    // stream.append_bit(true);
    // stream.append_byte(4);
    // stream.append_byte(5);
    // stream.append_byte(6);
    // stream.append_byte(7);
    // stream.append_byte(8);
    // stream.flush_to_file("test/binary_stream_test_file.bin").expect("TODO: panic message");
    // // let test = fs::read("binary_stream_test_file.bin").expect("TODO: panic message");
    println!("{:?}", stream);
}
