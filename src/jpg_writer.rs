use std::collections::HashMap;

use modinverse::egcd;
use nalgebra::SMatrix;

use crate::bit_stream::BitStream;
use crate::image::Image;
use crate::quantization;

/// Enum describing the different types of segments in a JPG file.
pub enum SegmentType {
    SOI,
    APP0,
    DQT,
    SOF0,
    DHT,
    SOS,
    EOI,
}

/// Write the given segment to the given stream, with data taken from the given image.
///
/// # Arguments
///
/// * `stream`: The BitStream to append the segment to.
/// * `image`: The image to take the data from.
/// * `segment_type`: The type of segment to write.
///
/// # Panics
/// * If the requested segment type isn't implemented yet.
pub fn write_segment_to_stream(stream: &mut BitStream, image: &Image, segment_type: SegmentType) {
    write_marker_for_segment(stream, &segment_type);
    match segment_type {
        SegmentType::SOI => (),
        SegmentType::APP0 => write_app0_segment(stream, image),
        SegmentType::SOF0 => write_sof0_segment(stream, image),
        SegmentType::SOS => write_sos_segment(stream),
        SegmentType::EOI => (),
        _ => panic!("Not implemented yet!"),
    };
}

fn write_marker_for_segment(stream: &mut BitStream, segment_type: &SegmentType) {
    stream.append::<u16>(match segment_type {
        SegmentType::SOI => 0xffd8,
        SegmentType::APP0 => 0xffe0,
        SegmentType::SOF0 => 0xffc0,
        SegmentType::EOI => 0xffd9,
        SegmentType::DHT => 0xffc4,
        SegmentType::DQT => 0xffdb,
        SegmentType::SOS => 0xffda,
    });
}

/// Write the APP0 segment of the JPG file.
/// This includes metadata regarding the version of the JFIF spec (in our case 1.1)
/// as well as data about the image and a potential preview image.
///
/// # Arguments
///
/// * `stream`: The BitStream to append the segment to.
/// * `image`: The image to take the data from.
fn write_app0_segment(stream: &mut BitStream, image: &Image) {
    // length of segment: 16
    stream.append::<u16>(16);
    // string "JFIF": 0x4a 0x46 0x49 0x46 0x00
    stream.append::<Vec<u8>>(vec![0x4a, 0x46, 0x49, 0x46, 0x00]); // TODO: use array rather than vec
                                                                  // revision number 1.1: 0x01 0x01
    stream.append::<u16>(0x0101);
    // of pixel size (0 => no unit, aspect ratio instead)
    stream.append::<u8>(0);
    // aspect ratio
    let (gcd, _1, _2) = egcd(image.width() as i32, image.height() as i32);
    let aspect_width = image.width() / gcd as u16;
    let aspect_height = image.height() / gcd as u16;
    stream.append(aspect_width);
    stream.append(aspect_height);
    // no thumbnail: 0x00 0x00
    stream.append::<u16>(0)
}

/// Write the SOF0 segment of the JPG file.
/// This includes metadata regarding the image compression.
///
/// # Arguments
///
/// * `stream`: The BitStream to append the segment to.
/// * `image`: The image to take the data from.
fn write_sof0_segment(stream: &mut BitStream, image: &Image) {
    // length, we always do coloured so 8 + 3*3
    stream.append::<u16>(17);
    // accuracy - we default to 8 as 12 and 16 aren't commonly supported
    stream.append::<u8>(8);
    // size
    stream.append(image.height());
    stream.append(image.width());
    // number of components - we always do coloured so 3
    stream.append::<u8>(3);

    let max_downsample_factor = std::cmp::max(
        std::cmp::max(image.y_downsample_factor(), image.cb_downsample_factor()),
        image.cr_downsample_factor(),
    ) as u8;
    // TODO: quantising tables, once they're implemented
    write_sof0_segment_component(
        stream,
        1, // id of the Y component.
        image.y_downsample_factor() as u8,
        false, // we don't downsample the Y component, ever
        0,
        max_downsample_factor,
    );
    write_sof0_segment_component(
        stream,
        2, // id of the Cb component.
        image.cb_downsample_factor() as u8,
        image.downsampled_vertically(),
        1,
        max_downsample_factor,
    );
    write_sof0_segment_component(
        stream,
        3, // id of the Cr component
        image.cr_downsample_factor() as u8,
        image.downsampled_vertically(),
        1,
        max_downsample_factor,
    );
}

/// Write a component in the SOF0 segment.
///
/// # Arguments
///
/// * `stream`: The BitStream to append the segment to.
/// * `id`: The ID of the component (1 for Y, 2 for Cb, 3 for Cr).
/// * `downsample_factor`: The factor by which the component was downsampled horizontally.
/// * `downsampled_vertically`: Whether the image was downsampled vertically.
/// * `quantise_table`: The quantise table.
fn write_sof0_segment_component(
    stream: &mut BitStream,
    id: u8,
    downsample_factor: u8,
    downsampled_vertically: bool,
    quantise_table: u8,
    max_downsample_factor: u8,
) {
    stream.append(id);
    // the four bits for vertical
    let mut downsample_value: u8 = if downsampled_vertically {
        max_downsample_factor / 2
    } else {
        max_downsample_factor
    };
    // the four bits for horizontal
    downsample_value += (max_downsample_factor / downsample_factor) << 4;
    stream.append(downsample_value);
    stream.append(quantise_table);
}

/// Write the SOS segment of the JPG file.
/// This denotes the start of the image data.
///
/// # Arguments
///
/// * `stream`: The BitStream to append the segment to.
/// * `image`: The image to take the data from.
fn write_sos_segment(stream: &mut BitStream) {
    // length, we always do coloured so 6 + 2*3
    stream.append::<u16>(12);
    // number of components, we always do coloured so 3
    stream.append::<u8>(3);
    // Y component - we use DHT 0 for its AC/DC
    stream.append::<u8>(1);
    stream.append::<u8>(0);
    // Cb component - we use DHT 1 for its AC/DC
    stream.append::<u8>(2);
    stream.append::<u8>(0b0001_0001);
    // Cr component - we use DHT 1 for its AC/DC
    stream.append::<u8>(3);
    stream.append::<u8>(0b0001_0001);
    // unused info for spectral/predictor selection
    // irrelevant for us because we don't do lossless, just write defaults
    stream.append::<u8>(0x00);
    stream.append::<u8>(0x3f);
    stream.append::<u8>(0x00);
}

pub fn write_dht_segment(
    stream: &mut BitStream,
    current_dht_id: u8,
    code_map: &HashMap<u8, (u8, u16)>,
    is_ac: bool,
) {
    write_marker_for_segment(stream, &SegmentType::DHT);
    let len: u16 = 19 + code_map.len() as u16;
    stream.append(len);
    let dht_info_byte = current_dht_id + (u8::from(is_ac) << 4);
    stream.append(dht_info_byte);

    for i in 1..17 {
        let amount: u8 = code_map.iter().filter(|val| val.1 .0 == i).count() as u8;
        stream.append(amount);
    }
    let mut code_vec: Vec<(&u8, &(u8, u16))> = code_map.iter().collect();

    code_vec.sort_by(|(_, code), (_2, code2)| {
        if code.0 == code2.0 {
            code.1.cmp(&code2.1)
        } else {
            code.0.cmp(&code2.0)
        }
    });

    for code in code_vec {
        stream.append(*code.0);
    }
}

/// Writes the DQT segment.
pub fn write_dqt_segment(stream: &mut BitStream, q_table: &SMatrix<f32, 8, 8>, number: u8) {
    write_marker_for_segment(stream, &SegmentType::DQT);
    stream.append(67u16);
    stream.append(number); // higher bits here would describe precision, but are always 0
    let zigzag = quantization::sample_zigzag(&q_table.map(|val| (1f32 / val).round() as u8));
    stream.append_many(&zigzag);
}

#[cfg(test)]
mod tests {
    use crate::bit_stream::BitStream;
    use crate::huffman::encode;
    use crate::jpg_writer::{
        write_app0_segment, write_dht_segment, write_marker_for_segment, write_segment_to_stream,
        write_sof0_segment, write_sof0_segment_component, SegmentType,
    };
    use crate::ppm_parser::read_ppm_from_file;
    use crate::quantization;

    use super::{write_dqt_segment, write_sos_segment};

    #[test]
    fn test_write_soi_marker_successful() {
        let mut stream = BitStream::open();
        write_marker_for_segment(&mut stream, &SegmentType::SOI);
        let data = vec![0xff, 0xd8];
        assert_eq!(data, *stream.data());
        assert_eq!(8, stream.bits_in_last_byte());
    }

    #[test]
    fn test_write_eoi_marker_successful() {
        let mut stream = BitStream::open();
        write_marker_for_segment(&mut stream, &SegmentType::EOI);
        let data = vec![0xff, 0xd9];
        assert_eq!(data, *stream.data());
        assert_eq!(8, stream.bits_in_last_byte());
    }

    #[test]
    fn test_write_app0_segment_successful() {
        let mut stream = BitStream::open();
        let image = read_ppm_from_file("test/valid_test_maxVal_15.ppm");
        write_app0_segment(&mut stream, &image);
        let data: Vec<u8> = vec![
            0, 16, 0x4a, 0x46, 0x49, 0x46, 0x00, 0x01, 0x01, 0, 0, 1, 0, 1, 0, 0,
        ];
        assert_eq!(data, *stream.data());
        assert_eq!(8, stream.bits_in_last_byte());
    }

    #[test]
    fn test_write_sof0_segment_component_downsampled_vertically_true_factor2() {
        let mut stream = BitStream::open();
        write_sof0_segment_component(&mut stream, 1, 2, true, 0, 2);
        let data: Vec<u8> = vec![1, 0x11, 0];
        assert_eq!(data, *stream.data());
        assert_eq!(8, stream.bits_in_last_byte());
    }

    #[test]
    fn test_write_sof0_segment_component_downsampled_vertically_false_factor2() {
        let mut stream = BitStream::open();
        write_sof0_segment_component(&mut stream, 1, 2, false, 0, 2);
        let data: Vec<u8> = vec![1, 0x12, 0];
        assert_eq!(data, *stream.data());
        assert_eq!(8, stream.bits_in_last_byte());
    }

    #[test]
    fn test_write_sof0_segment_component_downsampled_vertically_true_factor2_max4() {
        let mut stream = BitStream::open();
        write_sof0_segment_component(&mut stream, 1, 2, true, 0, 4);
        let data: Vec<u8> = vec![1, 0x22, 0];
        assert_eq!(data, *stream.data());
        assert_eq!(8, stream.bits_in_last_byte());
    }

    #[test]
    fn test_write_sof0_segment_component_downsampled_vertically_false_factor2_max4() {
        let mut stream = BitStream::open();
        write_sof0_segment_component(&mut stream, 1, 2, false, 0, 4);
        let data: Vec<u8> = vec![1, 0x24, 0];
        assert_eq!(data, *stream.data());
        assert_eq!(8, stream.bits_in_last_byte());
    }

    #[test]
    fn test_write_sof0_segment_no_downsampling() {
        let mut stream = BitStream::open();
        let image = read_ppm_from_file("test/valid_test_maxVal_15.ppm");
        write_sof0_segment(&mut stream, &image);
        let data: Vec<u8> = vec![0, 17, 8, 0, 4, 0, 4, 3, 1, 0x11, 0, 2, 0x11, 1, 3, 0x11, 1];
        assert_eq!(data, *stream.data());
        assert_eq!(8, stream.bits_in_last_byte());
    }

    #[test]
    fn test_write_sof0_segment_downsampling_4_2_0() {
        let mut stream = BitStream::open();
        let mut image = read_ppm_from_file("test/valid_test_maxVal_15.ppm");
        image.downsample(4, 2, 0);
        write_sof0_segment(&mut stream, &image);
        let data: Vec<u8> = vec![0, 17, 8, 0, 4, 0, 4, 3, 1, 0x22, 0, 2, 0x11, 1, 3, 0x11, 1];
        assert_eq!(data, *stream.data());
        assert_eq!(8, stream.bits_in_last_byte());
    }

    #[test]
    fn test_write_sos_segment() {
        let mut stream = BitStream::open();
        write_sos_segment(&mut stream);
        let expected_data: Vec<u8> = vec![0x00, 0x0c, 0x03, 0x01, 0x00, 0x02, 0b0001_0001, 0x03, 0b0001_0001, 0x00, 0x3f, 0x00];
        assert_eq!(&expected_data, stream.data());
    }

    #[test]
    fn test_write_whole_image_with_downsampling() {
        let mut stream = BitStream::open();
        let mut image = read_ppm_from_file("test/valid_test_maxVal_15.ppm");
        image.downsample(4, 2, 0);
        write_segment_to_stream(&mut stream, &image, SegmentType::SOI);
        write_segment_to_stream(&mut stream, &image, SegmentType::APP0);
        write_segment_to_stream(&mut stream, &image, SegmentType::SOF0);
        write_segment_to_stream(&mut stream, &image, SegmentType::EOI);
        let data: Vec<u8> = vec![
            0xff, 0xd8, 0xff, 0xe0, 0, 16, 0x4a, 0x46, 0x49, 0x46, 0x00, 0x01, 0x01, 0, 0, 1, 0, 1,
            0, 0, 0xff, 0xc0, 0, 17, 8, 0, 4, 0, 4, 3, 1, 0x22, 0, 2, 0x11, 1, 3, 0x11, 1, 0xff,
            0xd9,
        ];
        assert_eq!(data, *stream.data());
        assert_eq!(8, stream.bits_in_last_byte());
    }

    #[test]
    fn test_write_whole_image_without_downsampling() {
        let mut stream = BitStream::open();
        let image = read_ppm_from_file("test/valid_test_maxVal_15.ppm");
        write_segment_to_stream(&mut stream, &image, SegmentType::SOI);
        write_segment_to_stream(&mut stream, &image, SegmentType::APP0);
        write_segment_to_stream(&mut stream, &image, SegmentType::SOF0);
        write_segment_to_stream(&mut stream, &image, SegmentType::EOI);
        let data: Vec<u8> = vec![
            0xff, 0xd8, 0xff, 0xe0, 0, 16, 0x4a, 0x46, 0x49, 0x46, 0x00, 0x01, 0x01, 0, 0, 1, 0, 1,
            0, 0, 0xff, 0xc0, 0, 17, 8, 0, 4, 0, 4, 3, 1, 0x11, 0, 2, 0x11, 1, 3, 0x11, 1, 0xff,
            0xd9,
        ];
        assert_eq!(data, *stream.data());
        assert_eq!(8, stream.bits_in_last_byte());
    }

    #[test]
    fn test_write_dht_segment() {
        let mut symbol_stream = BitStream::open();
        for _ in 0..2 {
            symbol_stream.append_byte(1);
            symbol_stream.append_byte(2);
        }
        for _ in 0..3 {
            symbol_stream.append_byte(3);
            symbol_stream.append_byte(4);
        }
        for _ in 0..4 {
            symbol_stream.append_byte(5);
        }
        for _ in 0..5 {
            symbol_stream.append_byte(6);
        }

        for _ in 0..6 {
            symbol_stream.append_byte(7);
        }

        for _ in 0..7 {
            symbol_stream.append_byte(8);
        }
        for _ in 0..7 {
            symbol_stream.append_byte(9);
        }
        for _ in 0..7 {
            symbol_stream.append_byte(10);
        }
        for _ in 0..7 {
            symbol_stream.append_byte(11);
        }
        for _ in 0..7 {
            symbol_stream.append_byte(12);
        }
        for _ in 0..7 {
            symbol_stream.append_byte(13);
        }

        for _ in 0..7 {
            symbol_stream.append_byte(14);
        }
        for _ in 0..17 {
            symbol_stream.append_byte(15);
        }
        for _ in 0..71 {
            symbol_stream.append_byte(16);
        }
        for _ in 0..74 {
            symbol_stream.append_byte(17);
        }
        for _ in 0..17 {
            symbol_stream.append_byte(18);
        }
        for _ in 0..71 {
            symbol_stream.append_byte(19);
        }
        for _ in 0..74 {
            symbol_stream.append_byte(20);
        }
        for _ in 0..7 {
            symbol_stream.append_byte(21);
        }
        for _ in 0..7 {
            symbol_stream.append_byte(22);
        }
        for _ in 0..7 {
            symbol_stream.append_byte(23);
        }

        for _ in 0..7 {
            symbol_stream.append_byte(24);
        }
        for _ in 0..17 {
            symbol_stream.append_byte(25);
        }
        for _ in 0..71 {
            symbol_stream.append_byte(26);
        }
        for _ in 0..74 {
            symbol_stream.append_byte(27);
        }

        let (_, code_map) = encode(&mut symbol_stream);
        let mut stream = BitStream::open();
        write_dht_segment(&mut stream, 0, &code_map, false);
        let data: Vec<u8> = vec![
            0xff, 0xc4, // marker
            0, 46, // length
            0,  // HT information
            0,  // 1 bit codes
            0,  // 2 bit codes
            6,  // 3 bit codes
            0,  // 4 bit codes
            3,  // 5 bit codes
            4,  // 6 bit codes
            10, // 7 bit codes
            3,  // 8 bit codes
            1,  // 9 bit codes
            0, 0, 0, 0, 0, 0, 0, // remaining empty codes
            // symbols in order
            27, 20, 17, 26, 19, 16, 25, 18, 15, 24, 23, 22, 21, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5,
            4, 3, 2, 1,
        ];
        assert_eq!(data, *stream.data());
        assert_eq!(8, stream.bits_in_last_byte());
    }

    #[test]
    fn test_write_dqt_segment() {
        let mut stream = BitStream::open();
        let q_table = quantization::uniform_q_table(2f32);
        write_dqt_segment(&mut stream, &q_table, 1);

        let mut expected = BitStream::open();
        expected.append(0xffdb_u16);
        expected.append(67u16);
        expected.append(1u8);
        for _ in 0..64 {
            expected.append(2u8);
        }
        assert_eq!(expected, stream);
    }

    #[test]
    #[ignore]
    fn test_write_whole_image_4k_with_downsampling() {
        let mut stream = BitStream::open();
        let mut image = read_ppm_from_file("test/dwsample-ppm-4k.ppm");
        image.downsample(4, 2, 0);
        write_segment_to_stream(&mut stream, &image, SegmentType::SOI);
        write_segment_to_stream(&mut stream, &image, SegmentType::APP0);
        write_segment_to_stream(&mut stream, &image, SegmentType::SOF0);
        write_segment_to_stream(&mut stream, &image, SegmentType::EOI);
        //SOI
        let data: Vec<u8> = vec![
            0xff, 0xd8,
            //APP0: length 2 byte, JFIF0, major revision 1 byte, minor revision 1 byte, pixel ratio mode 1byte, x density 2 byte, y density 2 byte, thumbnail
            0xff, 0xe0, 0, 16, 0x4a, 0x46, 0x49, 0x46, 0x00, 0x01, 0x01, 0, 0, 16, 0, 9, 0, 0,
            //SOF0
            0xff, 0xc0, 0, 17, 8, 8, 112, 15, 0, 3, 1, 0x22, 0, 2, 0x11, 0, 3, 0x11, 0,
            //EOI
            0xff, 0xd9,
        ];
        assert_eq!(data, *stream.data());
        assert_eq!(8, stream.bits_in_last_byte());
    }
}
