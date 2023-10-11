use std::fs::{read, read_to_string};
use regex::Regex;

use crate::pixel::Pixel;

#[derive(Debug, PartialEq)]
pub struct Image {
    height: u16,
    width: u16,
    data: Vec<Vec<Pixel>>,
}

impl Image {}

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
        panic!("ALARM!");
    }
    let dimensions: Vec<_> = result[1].split(" ").collect();
    let maxValue = result[2].clone();
    let height: u16 = dimensions[0].parse().unwrap();
    let width: u16 =  dimensions[1].parse().unwrap();
    let re = Regex::new(r"\s+").unwrap();
    let mut image_values: Vec<Vec<Pixel>> = vec![];
    for i in 3..result.len() {
        let mut row: Vec<Pixel> = vec![];
        let values: Vec<String> = re.split(result[i].as_str()).map(|x| x.to_string()).collect();
        if values.len()/3 != width as usize {
            panic!("Line length to expected width mismatch");
        }
        // println!("{:?}", values);
        for j in (0..values.len()).step_by(3) {
            let pixel: Pixel = Pixel{one: values[j].parse().unwrap(), two: values[j + 1].parse().unwrap(), three: values[j + 2].parse().unwrap() };
            row.push(pixel);
        }
        image_values.push(row);
    }
    if image_values.len() != height as usize {
            panic!("row length to expected height mismatch");
    }
    Image{height, width, data: image_values}
}

    #[test]
    fn test_ppm_from_file_successful() {
        let read_image = read_ppm_from_file("test/valid_test.ppm");
        let expected_image = Image { height: 4, width: 4, data: vec![vec![Pixel { one: 0, two: 0, three: 0 }, Pixel { one: 0, two: 0, three: 0 }, Pixel { one: 0, two: 0, three: 0 }, Pixel { one: 15, two: 0, three: 15 }], vec![Pixel { one: 0, two: 0, three: 0 }, Pixel { one: 0, two: 15, three: 7 }, Pixel { one: 0, two: 0, three: 0 }, Pixel { one: 0, two: 0, three: 0 }], vec![Pixel { one: 0, two: 0, three: 0 }, Pixel { one: 0, two: 0, three: 0 }, Pixel { one: 0, two: 15, three: 7 }, Pixel { one: 0, two: 0, three: 0 }], vec![Pixel { one: 15, two: 0, three: 15 }, Pixel { one: 0, two: 0, three: 0 }, Pixel { one: 0, two: 0, three: 0 }, Pixel { one: 0, two: 0, three: 0 }]]};

        assert_eq!(expected_image, read_image);
    }

    #[test]
    #[should_panic]
    fn test_ppm_from_file_p3_not_present() {
        let read_image = read_ppm_from_file("test/invalid_test_p3_not_present.ppm");
    }

    #[test]
    #[should_panic]
    fn test_ppm_from_file_height_not_as_expected() {
        let read_image = read_ppm_from_file("test/invalid_test_width_not_equal_to_expected.ppm");
    }

    #[test]
    #[should_panic]
    fn test_ppm_from_file_width_not_as_expected() {
        let read_image = read_ppm_from_file("test/invalid_test_height_not_equal_to_expected.ppm");
    }