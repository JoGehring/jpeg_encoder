use crate::image::read_ppm_from_file;

mod image;
fn main() {
    let image = read_ppm_from_file("C:\\Users\\Nils\\IdeaProjects\\jpeg_encoder\\test\\valid_test.ppm");
    println!("{:?}", image);
}
