use std::fs::{read, read_to_string};
use regex::Regex;

#[derive(Debug, PartialEq)]
pub struct Image {
    height: u16,
    width: u16,
    data1: Vec<Vec<u16>>,
    data2: Vec<Vec<u16>>,
    data3: Vec<Vec<u16>>,
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

    let mut image_values1: Vec<Vec<u16>> = vec![];
    let mut image_values2: Vec<Vec<u16>> = vec![];
    let mut image_values3: Vec<Vec<u16>> = vec![];

    for i in 3..result.len() {
        let mut row1: Vec<u16> = vec![];
        let mut row2: Vec<u16> = vec![];
        let mut row3: Vec<u16> = vec![];
        let values: Vec<String> = re.split(result[i].as_str()).map(|x| x.to_string()).collect();
        if values.len()/3 != width as usize {
            panic!("Line length to expected width mismatch");
        }

        for j in (0..values.len()).step_by(3) {
            row1.push(values[j].parse().unwrap());
            row2.push(values[j+1].parse().unwrap());
            row3.push(values[j+2].parse().unwrap());
        }
        image_values1.push(row1);
        image_values2.push(row2);
        image_values3.push(row3);

    }

    if image_values1.len() != height as usize {
        panic!("row length to expected height mismatch");
    }
    if image_values2.len() != height as usize {
        panic!("row length to expected height mismatch");
    }
    if image_values3.len() != height as usize {
        panic!("row length to expected height mismatch");
    }

    Image{height, width, data1: image_values1, data2: image_values2, data3: image_values3}
}

    #[test]
    fn test_ppm_from_file_successful() {
        let read_image = read_ppm_from_file("test/valid_test.ppm");
        let expected_image = Image { height: 4, width: 4, data1: vec![vec![0, 0, 0, 15], vec![0, 0, 0, 0], vec![0, 0, 0, 0], vec![15, 0, 0, 0]], data2: vec![vec![0, 0, 0, 0], vec![0, 15, 0, 0], vec![0, 0, 15, 0], vec![0, 0, 0, 0]], data3: vec![vec![0, 0, 0, 15], vec![0, 7, 0, 0], vec![0, 0, 7, 0], vec![15, 0, 0, 0]] };

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
        let read_image = read_ppm_from_file("test/invalid_test_height_not_equal_to_expected.ppm");
    }

    #[test]
    #[should_panic]
    fn test_ppm_from_file_width_not_as_expected() {
        let read_image = read_ppm_from_file("test/invalid_test_width_not_equal_to_expected.ppm");
    }