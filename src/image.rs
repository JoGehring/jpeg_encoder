extern crate nalgebra as na;

use std::fs::read_to_string;

use na::{Matrix3, Vector3};
use regex::Regex;

#[derive(Debug, PartialEq)]
pub struct Image {
    height: u16,
    width: u16,
    data1: Vec<Vec<u16>>,
    data2: Vec<Vec<u16>>,
    data3: Vec<Vec<u16>>,
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

    return match result_as_int {
        Some(value) => (value[0], value[1], value[2]),
        None => panic!("Error while trying to convert to YCbCr!"),
    };
}

/// Reads an P3 PPM image file to image data structure
///
/// # Arguments
///
/// * `filename`: Path to the image file
///
/// # Examples
///
/// ```
/// let image = read_ppm_from_file("../path/to/image.ppm");
/// ```
///
/// # Panics
///
/// * PPM image file is not P3 format
/// * Any row or column of R/G/B values doesn't match the stated width and height
pub fn read_ppm_from_file(filename: &str) -> Image {
    let mut result: Vec<String> = vec![];
    for raw_line in read_to_string(filename).unwrap().lines() {
        let line = raw_line.to_string();
        if line.starts_with("#") {
            continue;
        }
        result.push(line);
    }
    if result[0] != String::from("P3") {
        panic!("Unsupported PPM format");
    }
    let dimensions: Vec<_> = result[1].split(" ").collect();
    // let maxValue = result[2].clone();
    let height: u16 = dimensions[0].parse().unwrap();
    let width: u16 = dimensions[1].parse().unwrap();
    let re = Regex::new(r"\s+").unwrap();

    let mut image_values1: Vec<Vec<u16>> = vec![];
    let mut image_values2: Vec<Vec<u16>> = vec![];
    let mut image_values3: Vec<Vec<u16>> = vec![];

    for i in 3..result.len() {
        let mut r_values: Vec<u16> = vec![];
        let mut g_values: Vec<u16> = vec![];
        let mut b_values: Vec<u16> = vec![];
        let values: Vec<String> = re
            .split(result[i].as_str())
            .map(|x| x.to_string())
            .collect();
        if values.len() / 3 != width as usize {
            panic!("Line length to expected width mismatch");
        }

        for j in (0..values.len()).step_by(3) {
            r_values.push(values[j].parse().unwrap());
            g_values.push(values[j + 1].parse().unwrap());
            b_values.push(values[j + 2].parse().unwrap());
        }
        image_values1.push(r_values);
        image_values2.push(g_values);
        image_values3.push(b_values);
    }

    if image_values1.len() != height as usize {
        panic!("R values row length to expected height mismatch");
    }
    if image_values2.len() != height as usize {
        panic!("G values row length to expected height mismatch");
    }
    if image_values3.len() != height as usize {
        panic!("B values row length to expected height mismatch");
    }

    Image {
        height,
        width,
        data1: image_values1,
        data2: image_values2,
        data3: image_values3,
    }
}

impl Image {
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
        if self.data1.len() != self.data2.len() || self.data2.len() != self.data3.len() {
            panic!("rgb_to_ycbcr called after downsampling!")
        }
        for row in 0..self.data1.len() {
            if self.data1[row].len() != self.data2[row].len()
                || self.data2[row].len() != self.data3[row].len()
            {
                panic!("rgb_to_ycbcr called after downsampling!")
            }
            for col in 0..self.data1[row].len() {
                let (y, cr, cb) = convert_rgb_values_to_ycbcr(
                    self.data1[row][col],
                    self.data2[row][col],
                    self.data3[row][col],
                );
                self.data1[row][col] = y;
                self.data2[row][col] = cr;
                self.data3[row][col] = cb;
            }
        }
    }

    pub fn downsample(&self, a: usize, b: usize, c: usize) {
        let product = a * b * c;
        if (product & product - 1) != 0 {
            panic!("One of the values is not in power of two");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{convert_rgb_values_to_ycbcr, read_ppm_from_file, Image};

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
            data1: Vec::from([Vec::from([0, 65535, 0, 0, 65535])]),
            data2: Vec::from([Vec::from([0, 0, 65535, 0, 65535])]),
            data3: Vec::from([Vec::from([0, 0, 0, 65535, 65535])]),
        };
        image.rgb_to_ycbcr();
        assert_eq!(
            image,
            Image {
                height: 1,
                width: 5,
                data1: Vec::from([Vec::from([0, 19595, 38469, 7471, 65535])]),
                data2: Vec::from([Vec::from([32767, 21711, 11062, 65535, 32774])]),
                data3: Vec::from([Vec::from([32767, 65535, 5334, 27439, 32774])]),
            }
        )
    }

    #[test]
    fn test_ppm_from_file_successful() {
        let read_image = read_ppm_from_file("test/valid_test.ppm");
        let expected_image = Image {
            height: 4,
            width: 4,
            data1: vec![
                vec![0, 0, 0, 15],
                vec![0, 0, 0, 0],
                vec![0, 0, 0, 0],
                vec![15, 0, 0, 0],
            ],
            data2: vec![
                vec![0, 0, 0, 0],
                vec![0, 15, 0, 0],
                vec![0, 0, 15, 0],
                vec![0, 0, 0, 0],
            ],
            data3: vec![
                vec![0, 0, 0, 15],
                vec![0, 7, 0, 0],
                vec![0, 0, 7, 0],
                vec![15, 0, 0, 0],
            ],
        };

        assert_eq!(expected_image, read_image);
    }

    #[test]
    #[should_panic]
    fn test_ppm_from_file_p3_not_present() {
        let _read_image = read_ppm_from_file("test/invalid_test_p3_not_present.ppm");
    }

    #[test]
    #[should_panic]
    fn test_ppm_from_file_height_not_as_expected() {
        let _read_image = read_ppm_from_file("test/invalid_test_height_not_equal_to_expected.ppm");
    }

    #[test]
    #[should_panic]
    fn test_ppm_from_file_width_not_as_expected() {
        let _read_image = read_ppm_from_file("test/invalid_test_width_not_equal_to_expected.ppm");
    }

    #[test]
    fn test_downsampling_parameters_are_power_of_two() {
        let image = read_ppm_from_file("test/valid_test.ppm");
        image.downsample(4, 2, 2);
    }

    #[test]
    #[should_panic]
    fn test_downsampling_a_value_not_power_of_two() {
        let image = read_ppm_from_file("test/valid_test.ppm");
        image.downsample(5, 2, 2);
    }

    #[test]
    #[should_panic]
    fn test_downsampling_b_value_not_power_of_two() {
        let image = read_ppm_from_file("test/valid_test.ppm");
        image.downsample(4, 3, 2);
    }

    #[test]
    #[should_panic]
    fn test_downsampling_c_value_not_power_of_two() {
        let image = read_ppm_from_file("test/valid_test.ppm");
        image.downsample(4, 2, 3);
    }
}
