use std::fs::read_to_string;

use regex::Regex;

use crate::image::{create_image, Image};

const SUPPORTED_FORMAT: &str = "P3";

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
    let result = parse_file_to_string_vec(filename);

    if result[0] != SUPPORTED_FORMAT {
        panic!("Unsupported PPM format");
    }

    let dimensions: Vec<_> = result[1].split(" ").collect();
    // let maxValue = result[2].clone();
    let height: u16 = dimensions[0].parse().unwrap();
    let width: u16 = dimensions[1].parse().unwrap();

    let (image_values1, image_values2, image_values3) =
        parse_image_values_from_string_array(result, width as usize, height as usize);

    if image_values1.len() != height as usize {
        panic!("R values row length to expected height mismatch");
    }
    if image_values2.len() != height as usize {
        panic!("G values row length to expected height mismatch");
    }
    if image_values3.len() != height as usize {
        panic!("B values row length to expected height mismatch");
    }

    create_image(height, width, image_values1, image_values2, image_values3)
}

/// Parse a file as a string vec.
///
/// # Arguments
///
/// * `filename`: The file name.
///
/// # Example
///
/// ```
/// let vector = parse_file_to_string_vec("/path/to/file");
/// ```
fn parse_file_to_string_vec(filename: &str) -> Vec<String> {
    let mut result: Vec<String> = vec![];
    for line in read_to_string(filename).unwrap().lines() {
        if line.starts_with("#") {
            continue;
        }
        result.push(line.to_owned());
    }
    result
}

/// Parse the image values from a string representation of a PPM file.
///
/// # Arguments
///
/// * `data`: The file to parse.
/// * `width`: The expected image width.
///
/// # Example
///
/// ```
/// let vector = parse_file_to_string_vec("/path/to/file");
/// // in a real world example, you should get the width from the file!
/// let (r, g, b) = parse_image_values_from_string_array(vector, 4);
/// ```
///
/// # Panics
///
/// * If the amount of values in any line doesn't match the expected width.
fn parse_image_values_from_string_array(
    data: Vec<String>,
    width: usize,
    height: usize,
) -> (Vec<Vec<u16>>, Vec<Vec<u16>>, Vec<Vec<u16>>) {
    let mut image_values1: Vec<Vec<u16>> = Vec::with_capacity(height);
    let mut image_values2: Vec<Vec<u16>> = Vec::with_capacity(height);
    let mut image_values3: Vec<Vec<u16>> = Vec::with_capacity(height);

    for i in 3..data.len() {
        let (r_values, g_values, b_values) = parse_image_values_from_line(&data[i], width);
        image_values1.push(r_values);
        image_values2.push(g_values);
        image_values3.push(b_values);
    }

    (image_values1, image_values2, image_values3)
}

/// Parse a line from a PPM file.
///
/// # Arguments
///
/// * `data`: The line to parse.
/// * `width`: The expected image width.
///
/// # Panics
///
/// * If the amount of values in the line doesn't match the expected width.
fn parse_image_values_from_line(data: &str, width: usize) -> (Vec<u16>, Vec<u16>, Vec<u16>) {
    // regex to split by whitespace
    let re = Regex::new(r"\s+").unwrap();
    let values: Vec<&str> = re.split(data).collect();

    if values.len() / 3 != width {
        panic!("Line length to expected width mismatch");
    }

    let mut r_values: Vec<u16> = Vec::with_capacity(width);
    let mut g_values: Vec<u16> = Vec::with_capacity(width);
    let mut b_values: Vec<u16> = Vec::with_capacity(width);

    for j in (0..values.len()).step_by(3) {
        r_values.push(values[j].parse().unwrap());
        g_values.push(values[j + 1].parse().unwrap());
        b_values.push(values[j + 2].parse().unwrap());
    }

    (r_values, g_values, b_values)
}

#[cfg(test)]
mod tests {
    use crate::image::create_image;

    use super::read_ppm_from_file;

// TODO: tests for utility functions

    #[test]
    fn test_ppm_from_file_successful() {
        let read_image = read_ppm_from_file("test/valid_test.ppm");
        let expected_image = create_image(
            4,
            4,
            vec![
                vec![0, 0, 0, 15],
                vec![0, 0, 0, 0],
                vec![0, 0, 0, 0],
                vec![15, 0, 0, 0],
            ],
            vec![
                vec![0, 0, 0, 0],
                vec![0, 15, 0, 0],
                vec![0, 0, 15, 0],
                vec![0, 0, 0, 0],
            ],
            vec![
                vec![0, 0, 0, 15],
                vec![0, 7, 0, 0],
                vec![0, 0, 7, 0],
                vec![15, 0, 0, 0],
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
    fn test_ppm_from_file_height_not_as_expected() {
        let _read_image = read_ppm_from_file("test/invalid_test_height_not_equal_to_expected.ppm");
    }

    #[test]
    #[should_panic]
    fn test_ppm_from_file_width_not_as_expected() {
        let _read_image = read_ppm_from_file("test/invalid_test_width_not_equal_to_expected.ppm");
    }
}