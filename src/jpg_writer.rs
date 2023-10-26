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
    EOI
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
    match segment_type {
        SegmentType::APP0 => write_app0_segment(stream, image),
        SegmentType::SOF0 => write_sof0_segment(stream, image),
        _ => panic!("Not implemented yet!")
    };
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
    // TODO
    panic!("Not implemented yet!")
}

/// Write the SOF0 segment of the JPG file.
/// This includes metadata regarding the image compression.
/// 
/// # Arguments
/// 
/// * `stream`: The BitStream to append the segment to.
/// * `image`: The image to take the data from.
fn write_sof0_segment(stream: &mut BitStream, image: &Image) {
    // TODO
    panic!("Not implemented yet!")
}