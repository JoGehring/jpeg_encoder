use na::{Matrix3, Vector3};

extern crate nalgebra as na;
#[derive(PartialEq)]
#[derive(Debug)]
struct Image {
    height: u16,
    width: u16,
    data1: Vec<Vec<u16>>,
    data2: Vec<Vec<u16>>,
    data3: Vec<Vec<u16>>,
}

const TRANSFORM_RGB_YCBCR_MATRIX: Matrix3<f32> = Matrix3::new(
    0.299, 0.587, 0.114, -0.1687, -0.3312, 0.5, 0.5, -0.4186, -0.0813,
);

fn convert_rgb_values_to_ycbcr(r: u16, g: u16, b: u16) -> (u16, u16, u16) {
    let mut result = TRANSFORM_RGB_YCBCR_MATRIX * Vector3::new(r as f32, g as f32, b as f32);

    result = Vector3::new(0.0, 32767.0, 32767.0) + result;

    let result_as_int = result.map(|value| value.round()).try_cast::<u16>();

    return match result_as_int {
        Some(value) => (value[0], value[1], value[2]),
        None => panic!("Error while trying to convert to YCbCr!"),
    };
}

impl Image {
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
}

#[cfg(test)]
mod tests {
    use super::{convert_rgb_values_to_ycbcr, Image};

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
        assert_eq!(image, Image {
            height: 1,
            width: 5,
            data1: Vec::from([Vec::from([0, 19595, 38469, 7471, 65535])]),
            data2: Vec::from([Vec::from([32767, 21711, 11062, 65535, 32774])]),
            data3: Vec::from([Vec::from([32767, 65535, 5334, 27439, 32774])]),
        })
    }
}
