#![allow(dead_code)]
// remove this once integrating - this is to avoid exessive and useless warnings for the time being

use ppm_parser::read_ppm_from_file;

/*
use crate::bit_stream::BitStream;
use crate::huffman::encode;
use crate::jpg_writer::write_dht_segment;
 */
mod image;
mod downsample;
mod ppm_parser;
mod bit_stream;
mod jpg_writer;
mod appendable_to_bit_stream;
mod huffman;
mod huffman_decoder;
mod utils;
mod package_merge;
mod dct;
mod arai;
mod parallel_dct;

fn main() {
    /*
    let mut image = read_ppm_from_file("test/dwsample-ppm-640.ppm");
    image.rgb_to_ycbcr();
    image.downsample(4, 2, 0);

    let mut target_stream = BitStream::open();
    jpg_writer::write_segment_to_stream(&mut target_stream, &image, jpg_writer::SegmentType::SOI);
    jpg_writer::write_segment_to_stream(&mut target_stream, &image, jpg_writer::SegmentType::APP0);
    jpg_writer::write_segment_to_stream(&mut target_stream, &image, jpg_writer::SegmentType::SOF0);

    let mut stream = BitStream::open();
    stream.append_byte(1);
    stream.append_byte(1);
    stream.append_byte(2);
    stream.append_byte(2);
    stream.append_byte(3);
    stream.append_byte(3);
    stream.append_byte(4);
    stream.append_byte(4);
    stream.append_byte(5);
    stream.append_byte(5);
    stream.append_byte(6);
    stream.append_byte(6);

    for _ in 0..7 {
        stream.append_byte(6);
    }
    for _ in 0..4 {
        stream.append_byte(3);
        stream.append_byte(4);
    }
    for _ in 0..2 {
        stream.append_byte(1);
        stream.append_byte(2);
    }
    for _ in 0..5 {
        stream.append_byte(5);
    }
    // let tree = parse_u8_stream(&mut stream);
    // println!("{:?}", tree);
    // package_merge_experimental(&mut stream, 3);
    let (_, code_map) = encode(&mut stream);
    write_dht_segment(&mut target_stream, 0, &code_map, false);

    jpg_writer::write_segment_to_stream(&mut target_stream, &image, jpg_writer::SegmentType::EOI);
    target_stream.flush_to_file("test/test_result.jpg");
    */

    let image = read_ppm_from_file("test/valid_test_8x8.ppm");
    let (y, cb, cr) = crate::parallel_dct::dct(&image);
    println!("{:?}", y);

    let (y_m, cb_m, cr_m) = image.to_matrices();
    println!("{:?}", dct::dct::arai_dct(&y_m[0]));
    println!("{:?}", dct::dct::arai_dct(&cb_m[0]));
    println!("{:?}", dct::dct::arai_dct(&cr_m[0]));
}
