use std::fs::read_to_string;

use lazy_static::lazy_static;
use regex::Regex;

use crate::image::{create_image, Image};

lazy_static! {
    static ref WHITESPACE_REGEX: Regex = Regex::new(r"\s+").unwrap();
}

const SUPPORTED_FORMAT: &str = "P3";

/// Reads an P3 PPM image file to image data structure.
/// If the width or height specified by the file is smaller than the actual width/height,
/// part of the data will be discarded.
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
/// * The PPM file is malformed so that image values contain non-numeric values.
/// * The width or height specified in the file is greater than the data's width/height.
pub fn read_ppm_from_file(filename: &str) -> Image {
    let result = parse_file_to_split_vec(filename);

    if result[0] != SUPPORTED_FORMAT {
        panic!("Unsupported PPM format");
    }

    let width: usize = result[1].parse().unwrap();
    let height: usize = result[2].parse().unwrap();

    let max_value_in_ppm: u16 = result[3].parse().unwrap();
    let scaling_factor = u16::MAX as f32 / max_value_in_ppm as f32;

    let (image_values1, image_values2, image_values3) = extract_pixel_values(&result, height, width, scaling_factor);

    create_image(height as u16, width as u16, image_values1, image_values2, image_values3)
}


/// Creates two-dimensional vectors with corresponding width and height for the three image data channels
///
/// # Arguments
///
/// * `raw_data`: The raw image data as row strings
/// * `height`: The image height given in the PPM file
/// * `width`: The image width given in the PPM file
/// * `scaling_factor`: The factor used to scale the image data to 65535
///
/// # Examples
///
/// ```
/// let (image_values1, image_values2, image_values3) = extract_pixel_values(&data, 1920, 1080, 3.14);
/// ```
fn extract_pixel_values(raw_data: &Vec<String>, height: usize, width: usize, scaling_factor: f32) -> (Vec<Vec<u16>>, Vec<Vec<u16>>, Vec<Vec<u16>>) {
    let mut image_values1: Vec<Vec<u16>> = vec![vec![0; width]; height];
    let mut image_values2: Vec<Vec<u16>> = vec![vec![0; width]; height];
    let mut image_values3: Vec<Vec<u16>> = vec![vec![0; width]; height];

    for i in 0..height {
        for j in 0..width {
            // index is 4 (because data starts at index 4)
            // plus width * 3 * i (to get to the row we're currently reading)
            // plus 3 * j (for the value in the row)
            let index = 4 + width * 3 * i + 3 * j;
            image_values1[i][j] = unwrap_and_scale(&raw_data[index], scaling_factor);
            image_values2[i][j] = unwrap_and_scale(&raw_data[index + 1], scaling_factor);
            image_values3[i][j] = unwrap_and_scale(&raw_data[index + 2], scaling_factor);
        }
    }
    (image_values1, image_values2, image_values3)
}

/// Parse the file and split it by white spaces/newlines.
/// Lines starting with '#' (comments) are discarded.
///
/// # Arguments
///
/// * `filename`: The file name.
///
/// # Example
///
/// ```
/// let my_vec = parse_file_to_split_vec("/path/to/file");
/// ```
fn parse_file_to_split_vec(filename: &str) -> Vec<String> {
    let string = parse_file_to_string(filename);
    WHITESPACE_REGEX.split(&string).map(|str_value| str_value.to_string()).collect()
}

/// Parse a file as a string.
/// Lines are connected with a blank space.
/// Lines starting with '#' (comments) are discarded.
///
/// # Arguments
///
/// * `filename`: The file name.
///
/// # Example
///
/// ```
/// let my_string = parse_file_to_string("/path/to/file");
/// ```
fn parse_file_to_string(filename: &str) -> String {
    let string = read_to_string(filename)
        .unwrap();
    let vec: Vec<_> = string
        .lines()
        .filter(|line| !line.starts_with("#"))
        .collect();
    vec.join(" ")
}

/// Apply the scaling factor. This is only extracted for readability purposes.
///
/// # Arguments
///
/// * `value`: The value to multiply with.
/// * `scaling_factor`: The factor to scale it by.
///
/// # Panics
///
/// * If the value cannot be parsed into a float.
fn unwrap_and_scale(value: &String, scaling_factor: f32) -> u16 {
    (value.parse::<f32>().unwrap() as f32 * scaling_factor) as u16
}

#[cfg(test)]
mod tests {
    use crate::image::create_image;

    use super::{extract_pixel_values, parse_file_to_split_vec, read_ppm_from_file, unwrap_and_scale};


    #[test]
    fn test_ppm_from_file_successful() {
        let read_image = read_ppm_from_file("test/valid_test_maxVal_15.ppm");
        let expected_image = create_image(
            4,
            4,
            vec![
                vec![0, 0, 0, 65535],
                vec![0, 0, 0, 0],
                vec![0, 0, 0, 0],
                vec![65535, 0, 0, 0],
            ],
            vec![
                vec![0, 0, 0, 0],
                vec![0, 65535, 0, 0],
                vec![0, 0, 65535, 0],
                vec![0, 0, 0, 0],
            ],
            vec![
                vec![0, 0, 0, 65535],
                vec![0, 30583, 0, 0],
                vec![0, 0, 30583, 0],
                vec![65535, 0, 0, 0],
            ],
        );

        assert_eq!(expected_image, read_image);
    }

    #[test]
    #[should_panic]
    fn test_ppm_from_file_p3_not_present() {
        let _read_image = read_ppm_from_file("test/invalid_test_p3_not_present.ppm");
    }

    #[test]
    #[should_panic]
    fn test_ppm_from_file_malformed() {
        let _read_image = read_ppm_from_file("test/invalid_test_malformed_value.ppm");
    }

    #[test]
    #[should_panic]
    fn test_ppm_from_file_too_large_height() {
        let _read_image = read_ppm_from_file("test/invalid_test_too_large_height.ppm");
    }

    #[test]
    #[should_panic]
    fn test_ppm_from_file_too_large_width() {
        let _read_image = read_ppm_from_file("test/invalid_test_too_large_width.ppm");
    }

    #[test]
    fn test_parse_file_to_split_vec_with_whitespace() {
        let data = parse_file_to_split_vec("test/string_test_file_with_whitespace.txt");
        assert_eq!(vec!["Hello", "this", "is", "a", "test"], data);
    }

    #[test]
    fn test_parse_file_to_split_vec_without_whitespace() {
        let data = parse_file_to_split_vec("test/string_test_file_without_whitespace.txt");
        assert_eq!(vec!["Hellothisisatest"], data);
    }

    #[test]
    fn test_extract_pixel_values_successful() {
        let mut data = Vec::with_capacity(16);
        data.push(String::from("P3"));
        data.push(String::from("2"));
        data.push(String::from("2"));
        data.push(String::from("15"));
        for i in 1..13 {
            data.push(i.to_string());
        }
        let (image_values1, image_values2, image_values3) = extract_pixel_values(&data, 2, 2, 3.14);
        assert_eq!(vec![vec![3, 12], vec![21, 31]], image_values1);
        assert_eq!(vec![vec![6, 15], vec![25, 34]], image_values2);
        assert_eq!(vec![vec![9, 18], vec![28, 37]], image_values3);
    }

    #[test]
    #[should_panic]
    fn test_extract_pixel_values_invalid_height() {
        let mut data = Vec::with_capacity(16);
        data.push(String::from("P3"));
        data.push(String::from("2"));
        data.push(String::from("2"));
        data.push(String::from("15"));
        for i in 1..13 {
            data.push(i.to_string());
        }
        extract_pixel_values(&data, 3, 2, 3.14);
    }

    #[test]
    #[should_panic]
    fn test_extract_pixel_values_invalid_width() {
        let mut data = Vec::with_capacity(16);
        data.push(String::from("P3"));
        data.push(String::from("2"));
        data.push(String::from("2"));
        data.push(String::from("15"));
        for i in 1..13 {
            data.push(i.to_string());
        }
        extract_pixel_values(&data, 2, 3, 3.14);
    }

    #[test]
    fn test_unwrap_and_scale_successful() {
        let calculated_value = unwrap_and_scale(&String::from("4"), 3.14);
        assert_eq!(12, calculated_value);
    }

    #[test]
    #[should_panic]
    fn test_unwrap_and_scale_invalid_string() {
        _ = unwrap_and_scale(&String::from("A"), 3.14);
    }
}
