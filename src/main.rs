use crate::ppm_parser::read_ppm_from_file;

mod image;
mod downsample;
mod ppm_parser;
mod bit_stream;

fn main() {
    let mut image = read_ppm_from_file("test/valid_test.ppm");
    println!("{:?}", image);
    image.rgb_to_ycbcr();
    println!("{:?}", image);
    image.downsample(4, 2, 0);
    println!("{:?}", image);
}
