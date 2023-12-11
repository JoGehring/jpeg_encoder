#![allow(dead_code)]
// remove this once integrating - this is to avoid exessive and useless warnings for the time being

use dct::DCTMode;
use parallel_dct::dct_single_channel;

use crate::image::create_image;

/*
use crate::bit_stream::BitStream;
use crate::huffman::encode;
use crate::jpg_writer::write_dht_segment;
 */
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
    /*
    // let mut image = read_ppm_from_file("test/dwsample-ppm-4k.ppm");
    let downsample_timer = std::time::Instant::now();
    image.downsample(4, 4, 0);
    println!("{}", downsample_timer.elapsed().as_millis());
    let dct_timer = std::time::Instant::now();
    parallel_dct::dct(&image);
    println!("{}", dct_timer.elapsed().as_millis());
    //parallel not-optimized/optimized: 147/31ms
    //non-parallel not-optimized/optimized: 1115/182ms
    */

    let mut image_data: Vec<Vec<u16>> = Vec::with_capacity(2160);
    for y in 0..2160 {
        image_data.push(Vec::with_capacity(3840));
        let index = image_data.len() - 1;
        let current_vec = &mut image_data[index];
        for x in 0..3840 {
            current_vec.push((x + y * 8) % 256);
        }
    }

    let image = create_image(2160, 3840, image_data, vec![], vec![]);
    let y_matrix = image.single_channel_to_matrices::<1>();

    for mode in [DCTMode::Arai, DCTMode::Direct, DCTMode::Matrix] {
        println!("Starting to test mode {}", mode);
        let timer_start = std::time::Instant::now();
        // 2000 should be plenty - would be enough for 2.5ms runs
        let mut times = Vec::with_capacity(4000);
        // do this for about 10 seconds
        while timer_start.elapsed().as_millis() < 10000 {
            let timer_single_run = std::time::Instant::now();
            let _ = parallel_dct::dct_matrix_vector(&y_matrix, &mode);
            times.push(timer_single_run.elapsed().as_millis());
        }
        times.sort();
        let min = times.first().unwrap();
        let max = times.last().unwrap();
        let mean: u128 = times.iter().sum::<u128>() / (times.len() as u128);
        let median = times[times.len() / 2];
        let twenty_fifth = times[times.len() / 4];
        let seventy_fifth = times[times.len() / 2 + times.len() / 4];
        println!("Mode ::::: Min ::: 25th ::: Median ::: 75th ::: Max ::::: Mean");
        println!(
            "{} ::::: {} ::: {} ::: {} ::: {} ::: {} ::::: {}",
            mode, min, twenty_fifth, median, seventy_fifth, max, mean
        );
    }

    // ppm --> dct --> idct --> ppm pipeline

    let image = ppm_parser::read_ppm_from_file("test/dwsample-ppm-4k.ppm");
    let (r_dct, g_dct, b_dct) = parallel_dct::dct(&image, &DCTMode::Matrix);
    let (ir_dct, ig_dct, ib_dct) = parallel_idct::idct(&r_dct, &g_dct, &b_dct);
    dct_to_ppm::to_ppm(
        (&ir_dct, &ig_dct, &ib_dct),
        image.height(),
        image.width(),
        "test/ppm_write.ppm",
    )
    .unwrap();
}
