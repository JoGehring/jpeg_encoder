extern crate nalgebra as na;

use na::{Matrix3, Vector3};

use crate::downsample::downsample_channel;

#[derive(Clone, Debug, PartialEq)]
pub struct Image {
    height: u16,
    width: u16,
    channel1: Vec<Vec<u16>>,
    channel2: Vec<Vec<u16>>,
    channel3: Vec<Vec<u16>>,
    downsample1: usize,
    downsample2: usize,
    downsample3: usize,
    downsampled_vertically: bool,
}

const TRANSFORM_RGB_YCBCR_MATRIX: Matrix3<f32> = Matrix3::new(
    0.299, 0.587, 0.114, -0.1687, -0.3312, 0.5, 0.5, -0.4186, -0.0813,
);

/// Convert an RGB value to a YCbCr value.
///
/// # Arguments
///
/// * `r`: The input's "Red" channel
/// * `g`: The input's "Green" channel
/// * `b`: The input's "Blue" channel
///
/// # Examples
///
/// ```
/// let color = convert_rgb_values_to_ycbcr(0, 0, 0);
/// assert_eq!(color, (0, 32767, 32767))
/// ```
///
/// # Panics
///
/// * Error casting back from floating point to integer numbers.
fn convert_rgb_values_to_ycbcr(r: u16, g: u16, b: u16) -> (u16, u16, u16) {
    let mut result = TRANSFORM_RGB_YCBCR_MATRIX * Vector3::new(r as f32, g as f32, b as f32);

    result = Vector3::new(0.0, 32767.0, 32767.0) + result;

    let result_as_int = result.map(|value| value.round()).try_cast::<u16>();

    match result_as_int {
        Some(value) => (value[0], value[1], value[2]),
        None => panic!("Error while trying to convert to YCbCr!"),
    }
}

/// Create an image.
///
/// # Arguments
///
/// * height: The image height.
/// * width: The image width.
/// * channel1: The first channel of data.
/// * channel2: The second channel of data.
/// * channel3: The third channel of data.
pub fn create_image(
    height: u16,
    width: u16,
    channel1: Vec<Vec<u16>>,
    channel2: Vec<Vec<u16>>,
    channel3: Vec<Vec<u16>>,
) -> Image {
    Image {
        height,
        width,
        channel1,
        channel2,
        channel3,
        ..Default::default()
    }
}

impl Image {
    /// Get the pixel at the x/y coordinates, with a bounds check.
    /// If it is outside the bounds, return the border pixel instead.
    ///
    /// # Arguments
    ///
    /// * `self`: This image
    /// * `x`: The x coordinate of the pixel.
    /// * `y`: The y coordinate of the pixel.
    ///
    /// # Examples
    /// ```
    /// let image = read_ppm_from_file("../path/to/image.ppm");
    /// println!('{}', image.pixel_at(4, 19));
    /// ```
    pub fn pixel_at(&self, x: u16, y: u16) -> (u16, u16, u16) {
        let mut actual_y = (std::cmp::max(y, 0)) as usize;
        actual_y = std::cmp::min(actual_y, self.channel1.len() - 1);
        let actual_y_downsampled = if self.downsampled_vertically {
            actual_y / 2
        } else {
            actual_y
        };

        let mut actual_x = (std::cmp::max(x, 0)) as usize;
        actual_x = std::cmp::min(actual_x, self.channel1[actual_y].len() - 1);
        let actual_x_1 = actual_x / self.downsample1;
        let actual_x_2 = actual_x / self.downsample2;
        let actual_x_3 = actual_x / self.downsample3;

        (
            self.channel1[actual_y][actual_x_1],
            self.channel2[actual_y_downsampled][actual_x_2],
            self.channel3[actual_y_downsampled][actual_x_3],
        )
    }

    /// Convert this image from RGB to YCbCr color space.
    ///
    /// # Arguments
    ///
    /// * `self`: This image
    ///
    /// # Examples
    ///
    /// ```
    /// let image = read_ppm_from_file("../path/to/image.ppm");
    /// image.rgb_to_ycbcr()
    /// ```
    ///
    /// # Panics
    ///
    /// * Method is called after the image was downsampled (the different channels aren't the same size)
    /// * Internal error when calling convert_rgb_values_to_ycbcr
    pub fn rgb_to_ycbcr(&mut self) {
        if self.downsample1 != 1
            || self.downsample2 != 1
            || self.downsample3 != 1
            || self.downsampled_vertically
        {
            panic!("rgb_to_ycbcr called after downsampling!")
        }
        for row in 0..self.channel1.len() {
            for col in 0..self.channel1[row].len() {
                let (y, cr, cb) = convert_rgb_values_to_ycbcr(
                    self.channel1[row][col],
                    self.channel2[row][col],
                    self.channel3[row][col],
                );
                self.channel1[row][col] = y;
                self.channel2[row][col] = cr;
                self.channel3[row][col] = cb;
            }
        }
    }

    /// Down-sample this image.
    /// `a`, `b` and `c` are expected to fit the segments of standard subsampling notation: https://en.wikipedia.org/wiki/Chroma_subsampling
    /// TODO: replace the above link with the proper RFC/place where the notation was defined
    ///
    /// # Arguments
    ///
    /// * `self`: This image
    /// * `a`: `a` as per the standard subsampling notation.
    /// * `b`: `b` as per the standard subsampling notation.
    /// * `c`: `c` as per the standard subsampling notation.
    ///
    /// # Examples
    /// ```
    /// let mut image = read_ppm_from_file("../path/to/image.ppm");
    /// image.downsample(4, 2, 2);
    /// ```
    /// # Panics
    ///
    /// * When a, b or c is not a power of two.
    pub fn downsample(&mut self, a: usize, b: usize, c: usize) {
        if a == b && a == c && b == c {
            return;
        }
        let product = (a * b * c) as isize;
        if (product & product - 1) != 0 {
            panic!("One of the values is not in power of two");
        }
        let result_cb = downsample_channel(&self.channel2, a, b, c == 0);
        let cr_b = if c == 0 { b } else { c };
        let result_cr = downsample_channel(&self.channel3, a, cr_b, c == 0);

        self.channel2 = result_cb;
        self.channel3 = result_cr;

        self.downsample2 *= a / b;
        self.downsample3 *= a / cr_b;
        self.downsampled_vertically |= c == 0;
    }
}

impl Default for Image {
    fn default() -> Image {
        Image {
            height: 0,
            width: 0,
            channel1: vec![],
            channel2: vec![],
            channel3: vec![],
            downsample1: 1,
            downsample2: 1,
            downsample3: 1,
            downsampled_vertically: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ppm_parser::read_ppm_from_file;

    use super::{convert_rgb_values_to_ycbcr, Image};

    #[test]
    fn test_downsample_image_factor_two() {
        let mut read_image = read_ppm_from_file("test/valid_test.ppm");
        read_image.downsample(4, 2, 2);
        assert_eq!(
            Image {
                width: 4,
                height: 4,
                channel1: vec![
                    vec![0, 0, 0, 15],
                    vec![0, 0, 0, 0],
                    vec![0, 0, 0, 0],
                    vec![15, 0, 0, 0],
                ],
                channel2: vec![vec![0, 0], vec![7, 0], vec![0, 7], vec![0, 0]],
                channel3: vec![vec![0, 7], vec![3, 0], vec![0, 3], vec![7, 0]],
                downsample1: 1,
                downsample2: 2,
                downsample3: 2,
                downsampled_vertically: false,
            },
            read_image
        );
    }

    #[test]
    fn test_downsample_image_no_downsample() {
        let mut read_image = read_ppm_from_file("test/valid_test.ppm");
        read_image.downsample(4, 4, 4);
        assert_eq!(
            Image {
                width: 4,
                height: 4,
                channel1: vec![
                    vec![0, 0, 0, 15],
                    vec![0, 0, 0, 0],
                    vec![0, 0, 0, 0],
                    vec![15, 0, 0, 0],
                ],
                channel2: vec![
                    vec![0, 0, 0, 0],
                    vec![0, 15, 0, 0],
                    vec![0, 0, 15, 0],
                    vec![0, 0, 0, 0],
                ],
                channel3: vec![
                    vec![0, 0, 0, 15],
                    vec![0, 7, 0, 0],
                    vec![0, 0, 7, 0],
                    vec![15, 0, 0, 0],
                ],
                downsample1: 1,
                downsample2: 1,
                downsample3: 1,
                downsampled_vertically: false,
            },
            read_image
        );
    }

    #[test]
    fn test_downsample_image_factor_four_and_vertical() {
        let mut read_image = read_ppm_from_file("test/valid_test.ppm");
        read_image.downsample(4, 1, 0);
        assert_eq!(
            Image {
                width: 4,
                height: 4,
                channel1: vec![
                    vec![0, 0, 0, 15],
                    vec![0, 0, 0, 0],
                    vec![0, 0, 0, 0],
                    vec![15, 0, 0, 0],
                ],
                channel2: vec![vec![1], vec![1]],
                channel3: vec![vec![2], vec![2]],
                downsample1: 1,
                downsample2: 4,
                downsample3: 4,
                downsampled_vertically: true,
            },
            read_image
        );
    }

    #[test]
    fn test_pixel_at_in_bounds() {
        let read_image = read_ppm_from_file("test/valid_test.ppm");
        let pixel = read_image.pixel_at(3, 0);
        assert_eq!((15, 0, 15), pixel);
    }

    #[test]
    fn test_pixel_at_x_out_of_bounds() {
        let read_image = read_ppm_from_file("test/valid_test.ppm");
        let pixel = read_image.pixel_at(4, 0);
        assert_eq!((15, 0, 15), pixel);
    }

    #[test]
    fn test_pixel_at_y_out_of_bounds() {
        let read_image = read_ppm_from_file("test/valid_test.ppm");
        let pixel = read_image.pixel_at(0, 4);
        assert_eq!((15, 0, 15), pixel);
    }

    #[test]
    fn test_pixel_at_y_and_x_out_of_bounds() {
        let read_image = read_ppm_from_file("test/valid_test.ppm");
        let pixel = read_image.pixel_at(4, 4);
        assert_eq!((0, 0, 0), pixel);
    }

    #[test]
    fn test_pixel_at_in_bounds_after_downsample() {
        let mut read_image = read_ppm_from_file("test/valid_test.ppm");
        read_image.downsample(4, 2, 2);
        let pixel = read_image.pixel_at(3, 0);
        assert_eq!((15, 0, 7), pixel);
    }

    #[test]
    fn test_pixel_at_x_out_of_bounds_after_downsample() {
        let mut read_image = read_ppm_from_file("test/valid_test.ppm");
        read_image.downsample(4, 2, 2);
        let pixel = read_image.pixel_at(4, 0);
        assert_eq!((15, 0, 7), pixel);
    }

    #[test]
    fn test_pixel_at_y_out_of_bounds_after_vertical_downsample() {
        let mut read_image = read_ppm_from_file("test/valid_test.ppm");
        read_image.downsample(4, 2, 0);
        let pixel = read_image.pixel_at(0, 4);
        assert_eq!((15, 0, 3), pixel);
    }

    #[test]
    fn test_pixel_at_y_and_x_out_of_bounds_after_downsample() {
        let mut read_image = read_ppm_from_file("test/valid_test.ppm");
        read_image.downsample(4, 2, 2);
        let pixel = read_image.pixel_at(4, 4);
        assert_eq!((0, 0, 0), pixel);
    }

    #[test]
    fn test_pixel_at_y_and_x_out_of_bounds_after_vertical_downsample() {
        let mut read_image = read_ppm_from_file("test/valid_test.ppm");
        read_image.downsample(4, 2, 0);
        let pixel = read_image.pixel_at(4, 4);
        assert_eq!((0, 3, 1), pixel);
    }

    fn test_convert_rgb_values_to_rcbcr_internal(start: (u16, u16, u16), target: (u16, u16, u16)) {
        let result = convert_rgb_values_to_ycbcr(start.0, start.1, start.2);
        assert_eq!(result, target);
    }

    #[test]
    fn test_convert_rgb_values_to_rcbcr_black() {
        test_convert_rgb_values_to_rcbcr_internal((0, 0, 0), (0, 32767, 32767));
    }

    #[test]
    fn test_convert_rgb_values_to_rcbcr_red() {
        test_convert_rgb_values_to_rcbcr_internal((65535, 0, 0), (19595, 21711, 65535))
    }

    #[test]
    fn test_convert_rgb_values_to_rcbcr_green() {
        test_convert_rgb_values_to_rcbcr_internal((0, 65535, 0), (38469, 11062, 5334))
    }

    #[test]
    fn test_convert_rgb_values_to_rcbcr_blue() {
        test_convert_rgb_values_to_rcbcr_internal((0, 0, 65535), (7471, 65535, 27439))
    }

    #[test]
    fn test_convert_rgb_values_to_rcbcr_white() {
        test_convert_rgb_values_to_rcbcr_internal((65535, 65535, 65535), (65535, 32774, 32774))
    }

    #[test]
    fn test_convert_rgb_to_rcbcr() {
        let mut image = Image {
            height: 1,
            width: 5,
            channel1: Vec::from([Vec::from([0, 65535, 0, 0, 65535])]),
            channel2: Vec::from([Vec::from([0, 0, 65535, 0, 65535])]),
            channel3: Vec::from([Vec::from([0, 0, 0, 65535, 65535])]),
            ..Default::default()
        };
        image.rgb_to_ycbcr();
        assert_eq!(
            image,
            Image {
                height: 1,
                width: 5,
                channel1: Vec::from([Vec::from([0, 19595, 38469, 7471, 65535])]),
                channel2: Vec::from([Vec::from([32767, 21711, 11062, 65535, 32774])]),
                channel3: Vec::from([Vec::from([32767, 65535, 5334, 27439, 32774])]),
                ..Default::default()
            }
        )
    }

    #[test]
    fn test_downsampling_parameters_are_power_of_two() {
        let mut image = read_ppm_from_file("test/valid_test.ppm");
        image.downsample(4, 2, 2);
    }

    #[test]
    #[should_panic]
    fn test_downsampling_a_value_not_power_of_two() {
        let mut image = read_ppm_from_file("test/valid_test.ppm");
        image.downsample(5, 2, 2);
    }

    #[test]
    #[should_panic]
    fn test_downsampling_b_value_not_power_of_two() {
        let mut image = read_ppm_from_file("test/valid_test.ppm");
        image.downsample(4, 3, 2);
    }

    #[test]
    #[should_panic]
    fn test_downsampling_c_value_not_power_of_two() {
        let mut image = read_ppm_from_file("test/valid_test.ppm");
        image.downsample(4, 2, 3);
    }
}
