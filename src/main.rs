use crate::bit_stream::BitStream;
use crate::ppm_parser::read_ppm_from_file;

mod image;
mod downsample;
mod ppm_parser;
mod bit_stream;
mod jpg_writer;

fn main() {
    // keep this in to avoid loads of "unused" warnings
    let mut image = read_ppm_from_file("test/valid_test_maxVal_15.ppm");
    println!("{:?}", image);
    image.rgb_to_ycbcr();
    println!("{:?}", image);
    image.downsample(4, 2, 0);
    println!("{:?}", image);

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
