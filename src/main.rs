#![allow(dead_code)]
// remove this once integrating - this is to avoid exessive and useless warnings for the time being

use crate::bit_stream::BitStream;
use crate::ppm_parser::read_ppm_from_file;

mod image;
mod downsample;
mod ppm_parser;
mod bit_stream;
mod jpg_writer;
mod appendable_to_bit_stream;
mod huffman;

fn main() {
    // keep this in to avoid loads of "unused" warnings
    let mut image = read_ppm_from_file("test/dwsample-ppm-4k.ppm");
    // println!("{:?}", image);
    image.rgb_to_ycbcr();
    // println!("{:?}", image);
    image.downsample(4, 2, 0);
    // println!("{:?}", image);

    // let mut stream = BitStream::open();
    // stream.append_bit(true);
    // stream.append_bit(true);
    // stream.append_bit(true);
    // stream.append_bit(true);
    // stream.append_bit(true);
    // stream.append_bit(true);
    // stream.append_bit(true);
    //
    // stream.append_byte(3);
    // stream.append_bit(true);
    // stream.append_byte(4);
    // stream.append_byte(5);
    // stream.append_byte(6);
    // stream.append_byte(7);
    // stream.append_byte(8);
    // stream.flush_to_file("test/binary_stream_test_file.bin").expect("TODO: panic message");
    // // let test = fs::read("binary_stream_test_file.bin").expect("TODO: panic message");
    // println!("{:?}", stream);

    let mut target_stream = BitStream::open();
    jpg_writer::write_segment_to_stream(&mut target_stream, &image, jpg_writer::SegmentType::SOI);
    jpg_writer::write_segment_to_stream(&mut target_stream, &image, jpg_writer::SegmentType::APP0);
    jpg_writer::write_segment_to_stream(&mut target_stream, &image, jpg_writer::SegmentType::SOF0);
    jpg_writer::write_segment_to_stream(&mut target_stream, &image, jpg_writer::SegmentType::EOI);
    target_stream.flush_to_file("test/test4k.jpg").expect("TODO: Panic message");

}
