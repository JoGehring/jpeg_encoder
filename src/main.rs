#![allow(dead_code)]
// remove this once integrating - this is to avoid exessive and useless warnings for the time being

use ppm_parser::read_ppm_from_file;
use scoped_threadpool::Pool;
use dct::DCTMode;

use crate::image::create_image;
use crate::utils::THREAD_COUNT;

use crate::bit_stream::BitStream;
use crate::huffman::encode;
mod quantization;
mod appendable_to_bit_stream;
mod arai;
mod bit_stream;
mod dct;
mod dct_to_ppm;
mod downsample;
mod huffman;
mod huffman_decoder;
mod image;
mod jpg_writer;
mod package_merge;
mod parallel_dct;
mod parallel_downsample;
mod parallel_idct;
mod ppm_parser;
mod utils;
mod dct_constant_calculator;
mod dct_constants;

fn main() {
    let mut pool = Pool::new(*THREAD_COUNT as u32);

    let mut image = read_ppm_from_file("test/dwsample-ppm-640.ppm");
    image.rgb_to_ycbcr();
    image.downsample(4, 2, 0);

    let (y_dct, cb_dct, cr_dct) = parallel_dct::dct(&image, &DCTMode::Arai, &mut pool);

    // TODO: Quantize
    // TODO: Zigzag
    // TODO: RLE/Category encoding
    // TODO: Huffman
    /*
    let (_, code_map) = encode(&mut stream);
    jpg_writer::write_dht_segment(&mut target_stream, 0, &code_map, false);
    */

    let mut target_stream = BitStream::open();
    jpg_writer::write_segment_to_stream(&mut target_stream, &image, jpg_writer::SegmentType::SOI);
    jpg_writer::write_segment_to_stream(&mut target_stream, &image, jpg_writer::SegmentType::APP0);
    // TODO: DQT
    jpg_writer::write_segment_to_stream(&mut target_stream, &image, jpg_writer::SegmentType::SOF0);
    // TODO: DHT
    // TODO: SOS
    // TODO: Image data

    jpg_writer::write_segment_to_stream(&mut target_stream, &image, jpg_writer::SegmentType::EOI);

    target_stream.flush_to_file("test/test_result.jpg");
}
