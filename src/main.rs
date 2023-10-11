use crate::image::read_ppm_from_file;

mod image;
mod pixel;
fn main() {
    let image = read_ppm_from_file("/Users/jogehring/Downloads/boxes_1.ppm");
    println!("{:?}", image);
}
