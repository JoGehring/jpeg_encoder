use std::collections::HashMap;

use modinverse::egcd;

use crate::bit_stream::BitStream;
use crate::image::Image;

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
        SegmentType::SOI => return,
        SegmentType::APP0 => write_app0_segment(stream, image),
        SegmentType::SOF0 => write_sof0_segment(stream, image),
        SegmentType::EOI => return,
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
        _ => panic!("Not implemented yet!"),
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

    let max_downsample_factor = std::cmp::max(std::cmp::max(image.y_downsample_factor(), image.cb_downsample_factor()), image.cr_downsample_factor()) as u8;
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
        0,
        max_downsample_factor,
    );
    write_sof0_segment_component(
        stream,
        3, // id of the Cr component
        image.cr_downsample_factor() as u8,
        image.downsampled_vertically(),
        0,
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
    let mut downsample_value: u8 = if downsampled_vertically { max_downsample_factor / 2 } else { max_downsample_factor };
    // the four bits for horizontal
    downsample_value += max_downsample_factor / downsample_factor << 4;
    stream.append(downsample_value);
    stream.append(quantise_table);
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
        let amount: u8 = code_map.iter().filter(|val| val.1.0 == i).count() as u8;
        stream.append(amount);
    }
    let mut code_vec: Vec<(&u8, &(u8, u16))> = code_map.iter().collect();
    code_vec.sort_by(|(_, code), (_2, code2)|
        { return if code.0 == code2.0 { code.1.cmp(&code2.1) } else { code.0.cmp(&code2.0) }; });
    for code in code_vec {
        stream.append(*code.0);
    }
}


#[cfg(test)]
mod tests {
    use crate::bit_stream::BitStream;
    use crate::huffman::encode;
    use crate::jpg_writer::{SegmentType, write_app0_segment, write_dht_segment, write_marker_for_segment, write_segment_to_stream, write_sof0_segment, write_sof0_segment_component};
    use crate::ppm_parser::read_ppm_from_file;

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
        let data: Vec<u8> = vec![0, 16, 0x4a, 0x46, 0x49, 0x46, 0x00, 0x01, 0x01, 0, 0, 1, 0, 1, 0, 0];
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
        let data: Vec<u8> = vec![0, 17, 8, 0, 4, 0, 4, 3, 1, 0x11, 0, 2, 0x11, 0, 3, 0x11, 0];
        assert_eq!(data, *stream.data());
        assert_eq!(8, stream.bits_in_last_byte());
    }

    #[test]
    fn test_write_sof0_segment_downsampling_4_2_0() {
        let mut stream = BitStream::open();
        let mut image = read_ppm_from_file("test/valid_test_maxVal_15.ppm");
        image.downsample(4, 2, 0);
        write_sof0_segment(&mut stream, &image);
        let data: Vec<u8> = vec![0, 17, 8, 0, 4, 0, 4, 3, 1, 0x22, 0, 2, 0x11, 0, 3, 0x11, 0];
        assert_eq!(data, *stream.data());
        assert_eq!(8, stream.bits_in_last_byte());
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
        let data: Vec<u8> = vec![0xff, 0xd8, 0xff, 0xe0, 0, 16, 0x4a, 0x46, 0x49, 0x46, 0x00, 0x01, 0x01, 0, 0, 1, 0, 1, 0, 0, 0xff, 0xc0, 0, 17, 8, 0, 4, 0, 4, 3, 1, 0x22, 0, 2, 0x11, 0, 3, 0x11, 0, 0xff, 0xd9];
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
        let data: Vec<u8> = vec![0xff, 0xd8, 0xff, 0xe0, 0, 16, 0x4a, 0x46, 0x49, 0x46, 0x00, 0x01, 0x01, 0, 0, 1, 0, 1, 0, 0, 0xff, 0xc0, 0, 17, 8, 0, 4, 0, 4, 3, 1, 0x11, 0, 2, 0x11, 0, 3, 0x11, 0, 0xff, 0xd9];
        assert_eq!(data, *stream.data());
        assert_eq!(8, stream.bits_in_last_byte());
    }

    // #[test]
    // fn test_write_dht_segment() {
    //     let mut stream = BitStream::open();
    //
    //     let (mut stream, code_map) = encode(&mut stream);
    //     write_dht_segment(&mut stream, 0, &code_map, false);
    //     let data: Vec<u8> = vec![0xff, 0xc4, 0xff, 0xe0, 0, 16, 0x4a, 0x46, 0x49, 0x46, 0x00, 0x01, 0x01, 0, 0, 1, 0, 1, 0, 0, 0xff, 0xc0, 0, 17, 8, 0, 4, 0, 4, 3, 1, 0x11, 0, 2, 0x11, 0, 3, 0x11, 0, 0xff, 0xd9];
    //     assert_eq!(data, *stream.data());
    //     assert_eq!(8, stream.bits_in_last_byte());
    // }

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
        let data: Vec<u8> = vec![0xff, 0xd8,
                                 //APP0: length 2 byte, JFIF0, major revision 1 byte, minor revision 1 byte, pixel ratio mode 1byte, x density 2 byte, y density 2 byte, thumbnail
                                 0xff, 0xe0, 0, 16, 0x4a, 0x46, 0x49, 0x46, 0x00, 0x01, 0x01, 0, 0, 16, 0, 9, 0, 0,
                                 //SOF0
                                 0xff, 0xc0, 0, 17, 8, 8, 112, 15, 0, 3, 1, 0x22, 0, 2, 0x11, 0, 3, 0x11, 0,
                                 //EOI
                                 0xff, 0xd9];
        assert_eq!(data, *stream.data());
        assert_eq!(8, stream.bits_in_last_byte());
    }
}