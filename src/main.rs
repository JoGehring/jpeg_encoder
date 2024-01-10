#![allow(dead_code)]
// remove this once integrating - this is to avoid exessive and useless warnings for the time being

use scoped_threadpool::Pool;

use dct::DCTMode;
use image_data_writer::write_image_data_to_stream;
use ppm_parser::read_ppm_from_file;

use crate::bit_stream::BitStream;
use crate::utils::THREAD_COUNT;

mod appendable_to_bit_stream;
mod arai;
mod bit_stream;
mod coefficient_encoder;
mod dct;
mod dct_constant_calculator;
mod dct_constants;
mod dct_to_ppm;
mod downsample;
mod huffman;
mod huffman_decoder;
mod image;
mod image_data_writer;
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

    // let mut image = read_ppm_from_file("test/dwsample-ppm-1920.ppm");
    // let mut image = read_ppm_from_file("test/dwsample-ppm-4k.ppm");
    let mut image = read_ppm_from_file("test/test_16x16_red.ppm");

    image.rgb_to_ycbcr();
    image.downsample(4, 2, 0);

    let (mut y_dct, mut cb_dct, mut cr_dct) = parallel_dct::dct(&image, &DCTMode::Arai, &mut pool);

    let luminance_q_table = quantization::box_q_table(1f32, 3, 1f32);
    let chrominance_q_table = quantization::box_q_table(2f32, 3, 1f32);

    let y_quant = parallel_quantize::quantize_zigzag(&mut y_dct, luminance_q_table, &mut pool);
    let cb_quant = parallel_quantize::quantize_zigzag(&mut cb_dct, chrominance_q_table, &mut pool);
    let cr_quant = parallel_quantize::quantize_zigzag(&mut cr_dct, chrominance_q_table, &mut pool);

    let mut y_dc = coefficient_encoder::dc_coefficients(&y_quant);
    let cb_dc = coefficient_encoder::dc_coefficients(&cb_quant);
    let cr_dc = coefficient_encoder::dc_coefficients(&cr_quant);

    let mut y_ac = coefficient_encoder::ac_coefficients(&y_quant);
    let cb_ac = coefficient_encoder::ac_coefficients(&cb_quant);
    let cr_ac = coefficient_encoder::ac_coefficients(&cr_quant);

    coefficient_encoder::reorder_y_coefficients(&mut y_dc, image.width());
    coefficient_encoder::reorder_y_coefficients(&mut y_ac, image.width());

    let (y_dc_encoded, huffman_dc_y) = coefficient_encoder::encode_dc_coefficients(&y_dc);
    let (cbcr_dc_encoded, huffman_dc_cbcr) = coefficient_encoder::encode_two_dc_coefficients(&cb_dc, &cr_dc);
    let cb_dc_encoded = &cbcr_dc_encoded[0..cbcr_dc_encoded.len() / 2];
    let cr_dc_encoded = &cbcr_dc_encoded[(cbcr_dc_encoded.len() / 2)..cbcr_dc_encoded.len()];

    let (y_ac_encoded, huffman_ac_y) = coefficient_encoder::encode_ac_coefficients(&y_ac);
    let (cbcr_ac_encoded, huffman_ac_cbcr) = coefficient_encoder::encode_two_ac_coefficients(&cb_ac, &cr_ac);
    let cb_ac_encoded = &cbcr_ac_encoded[0..cbcr_ac_encoded.len() / 2];
    let cr_ac_encoded = &cbcr_ac_encoded[(cbcr_ac_encoded.len() / 2)..cbcr_ac_encoded.len()];
    // println!("{:?}", y_ac_encoded);
    // println!("{:?}", huffman_ac_y);
    // println!("{:?}", cb_ac_encoded);
    // println!("{:?}", cr_ac_encoded);
    // println!("{:?}", huffman_ac_cbcr);


    let mut target_stream = BitStream::open();
    jpg_writer::write_segment_to_stream(&mut target_stream, &image, jpg_writer::SegmentType::SOI);
    jpg_writer::write_segment_to_stream(&mut target_stream, &image, jpg_writer::SegmentType::APP0);
    jpg_writer::write_dqt_segment(&mut target_stream, &luminance_q_table, 0);
    jpg_writer::write_dqt_segment(&mut target_stream, &chrominance_q_table, 1);
    jpg_writer::write_segment_to_stream(&mut target_stream, &image, jpg_writer::SegmentType::SOF0);
    jpg_writer::write_dht_segment(&mut target_stream, 0, &huffman_dc_y, false);
    jpg_writer::write_dht_segment(&mut target_stream, 1, &huffman_dc_cbcr, false);

    jpg_writer::write_dht_segment(&mut target_stream, 2, &huffman_ac_y, true);
    jpg_writer::write_dht_segment(&mut target_stream, 3, &huffman_ac_cbcr, true);
    jpg_writer::write_segment_to_stream(&mut target_stream, &image, jpg_writer::SegmentType::SOS);

    target_stream.byte_stuffing(true);
    write_image_data_to_stream(&mut target_stream, &y_dc_encoded, cb_dc_encoded, cr_dc_encoded, &y_ac_encoded, cb_ac_encoded, cr_ac_encoded);
    target_stream.byte_stuffing(false);

    target_stream.pad_last_byte(true);

    jpg_writer::write_segment_to_stream(&mut target_stream, &image, jpg_writer::SegmentType::EOI);

    target_stream.flush_to_file("output.jpg");
}
