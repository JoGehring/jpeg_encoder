#![allow(dead_code)]
// remove this once integrating - this is to avoid exessive and useless warnings for the time being

use dct::DCTMode;
use ppm_parser::read_ppm_from_file;
use scoped_threadpool::Pool;

use crate::image::create_image;
use crate::utils::THREAD_COUNT;

use crate::bit_stream::BitStream;
use crate::huffman::encode;
mod appendable_to_bit_stream;
mod arai;
mod bit_stream;
mod dct;
mod dct_constant_calculator;
mod dct_constants;
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
mod parallel_quantize;
mod ppm_parser;
mod quantization;
mod utils;

fn main() {
    let mut pool = Pool::new(*THREAD_COUNT as u32);

    let mut image = read_ppm_from_file("test/dwsample-ppm-1920.ppm");
    image.rgb_to_ycbcr();
    image.downsample(4, 2, 0);

    let (mut y_dct, mut cb_dct, mut cr_dct) = parallel_dct::dct(&image, &DCTMode::Arai, &mut pool);
    // TODO: different q_tables?
    let luminance_q_table = quantization::uniform_q_table(1f32);
    let chrominance_q_table = quantization::uniform_q_table(2f32);
    let y_quant = parallel_quantize::quantize_zigzag(&mut y_dct, luminance_q_table, &mut pool);
    let cb_quant = parallel_quantize::quantize_zigzag(&mut cb_dct, chrominance_q_table, &mut pool);
    let cr_quant = parallel_quantize::quantize_zigzag(&mut cr_dct, chrominance_q_table, &mut pool);

    // TODO: RLE/Category encoding
    // TODO: Huffman
    /*
    let (_, code_map) = encode(&mut stream);
    jpg_writer::write_dht_segment(&mut target_stream, 0, &code_map, false);
    */

    let mut target_stream = BitStream::open();
    jpg_writer::write_segment_to_stream(&mut target_stream, &image, jpg_writer::SegmentType::SOI);
    jpg_writer::write_segment_to_stream(&mut target_stream, &image, jpg_writer::SegmentType::APP0);
    jpg_writer::write_dqt_segment(&mut target_stream, &luminance_q_table, 0);
    jpg_writer::write_dqt_segment(&mut target_stream, &chrominance_q_table, 1);
    jpg_writer::write_segment_to_stream(&mut target_stream, &image, jpg_writer::SegmentType::SOF0);
    // TODO: DHT
    // TODO: SOS
    // TODO: Image data

    jpg_writer::write_segment_to_stream(&mut target_stream, &image, jpg_writer::SegmentType::EOI);

    target_stream.flush_to_file("output.jpg");
}
