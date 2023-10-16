use crate::image::read_ppm_from_file;

mod image;
mod downsample;

fn main() {
    let mut image = read_ppm_from_file("test/valid_test.ppm");
    println!("{:?}", image);
    image.downsample(4, 2, 0);
    println!("{:?}", image);
}
