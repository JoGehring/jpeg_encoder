extern crate nalgebra as na;

use std::sync::mpsc::{self, Receiver};
use std::thread::{self, JoinHandle};

use na::{Matrix3, SMatrix, Vector3};

use crate::downsample::downsample_channel;
use crate::parallel_downsample;
use crate::utils::THREAD_COUNT;

/// Image data structure for parsed image files
///
/// # Attributes
///
/// * `height`: Image height in pixels
/// * `width`: Image width in pixels
/// * `channel`: The three channels for pixel data, either RGB or YCbCr in this order 1-3
/// * `downsample_factors`: The factor of downsampling for the corresponding channels, 1 by default.
/// E.g. for 4:2:0 the downsampling factor for Cb and Cr is 2, because we only keep every second value
/// * `downsampled_vertically`: True if two rows have been combined (e.g. for 4:2:0)
#[derive(Clone, Debug, PartialEq)]
pub struct Image {
    height: u16,
    width: u16,
    channel1: Vec<Vec<i16>>,
    channel2: Vec<Vec<i16>>,
    channel3: Vec<Vec<i16>>,
    y_downsample_factor: usize,
    cb_downsample_factor: usize,
    cr_downsample_factor: usize,
    downsampled_vertically: bool,
}

const TRANSFORM_RGB_YCBCR_MATRIX: Matrix3<f32> = Matrix3::new(
    0.299, 0.587, 0.114, -0.1687, -0.3312, 0.5, 0.5, -0.4186, -0.0813,
);

const RGB_TO_YCBCR_OFFSET: Vector3<f32> = Vector3::new(0.0, 127.0, 127.0);
const RGB_HALF_OFFSET: Vector3<f32> = Vector3::new(127.0, 127.0, 127.0);

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
/// assert_eq!(color, (0, 127, 127))
/// ```
///
/// # Panics
///
/// * Error casting back from floating point to integer numbers.
fn convert_rgb_values_to_ycbcr(r: i16, g: i16, b: i16) -> (i16, i16, i16) {
    let mut result = TRANSFORM_RGB_YCBCR_MATRIX * Vector3::new(r as f32, g as f32, b as f32);

    result += RGB_TO_YCBCR_OFFSET;
    result -= RGB_HALF_OFFSET;
    let result_as_int = result.map(|value| value.round()).try_cast::<i16>();

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
    channel1: Vec<Vec<i16>>,
    channel2: Vec<Vec<i16>>,
    channel3: Vec<Vec<i16>>,
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

/// Convert one channel into a Vec of 8x8 matrices containing its data.
/// This assumes the channel's dimensions can be divided by 8!
///
/// # Arguments
/// * `channel`: The channel for which data should be converted.
/// * `downsample_factor`: The factor by which the channel was downsampled horizontally.
/// * `downsampled_vertically`: Whether the channel was downsampled vertically.
///
/// # Panics
/// * If `channel`'s dimensions aren't divisible by 8.
fn channel_to_matrices(channel: &Vec<Vec<i16>>) -> Vec<SMatrix<f32, 8, 8>> {
    let mut chunk_size = channel.len() / *THREAD_COUNT;
    // always ensure that chunk size is divisible by 8 - otherwise threads don't get proper number of rows
    chunk_size += 8 - chunk_size % 8;
    let chunks: std::slice::Chunks<'_, Vec<i16>> = channel.chunks(chunk_size);
    let mut handles: Vec<JoinHandle<()>> = Vec::with_capacity(*THREAD_COUNT);
    let mut receivers: Vec<Receiver<Vec<SMatrix<f32, 8, 8>>>> = Vec::with_capacity(*THREAD_COUNT);

    for chunk in chunks {
        let (tx, rx) = mpsc::channel();
        // slow copy because directly using `chunk` leads to borrow issues. maybe fixable with lifetimes?
        let chunk_owned = chunk.to_owned();
        let handle = thread::spawn(move || {
            let mut result_vec: Vec<SMatrix<f32, 8, 8>> =
                Vec::with_capacity((chunk_owned.len() / 8) * (chunk_owned[0].len() / 8));
            for y in (0..chunk_owned.len()).step_by(8) {
                append_row_matrices_to_channel_matrix(&chunk_owned, y, &mut result_vec);
            }
            tx.send(result_vec).unwrap()
        });

        handles.push(handle);
        receivers.push(rx);
    }

    let mut result: Vec<SMatrix<f32, 8, 8>> =
        Vec::with_capacity((channel.len() / 8) * (channel[0].len() / 8));
    for handle in handles {
        handle.join().unwrap();
    }
    for receiver in receivers {
        result.extend(receiver.recv().unwrap());
    }
    result
}

/// Convert 8 rows' worth of a channel's data into a Vec of 8x8 matrices containing that data.
/// This assumes the channel's width can be divided by 8!
///
/// # Arguments
/// * `channel`: The channel for which data should be converted.
/// * `y`: The y index of the first of the 8 rows.
/// * `downsample_factor`: The factor by which the channel was downsampled horizontally.
/// * `downsampled_vertically`: Whether the channel was downsampled vertically.
/// * `result_vec`: The Vec to append the resulting matrices to.
///
/// # Panics
/// * If `channel`'s width is't divisible by 8.
#[inline(always)]
fn append_row_matrices_to_channel_matrix(
    channel: &[Vec<i16>],
    y: usize,
    result_vec: &mut Vec<SMatrix<f32, 8, 8>>,
) {
    let row_vectors = &channel[y..y + 8];
    for x in (0..channel[0].len()).step_by(8) {
        append_matrix_at_coordinates_to_channel_matrix(x, row_vectors, result_vec);
    }
}

/// Convert a row of 8 values in row_vectors into a 8x8 matrix.
/// This assumes the channel's width can be divided by 8!
///
/// # Arguments
/// * `x`: The x index of the first of the 8 values in each row.
/// * `row_vectors`: The vectors to take data from. This should always have the size 8, although it isn't checked.
/// * `downsample_factor`: The factor by which the channel was downsampled horizontally.
/// * `downsampled_vertically`: Whether the channel was downsampled vertically.
/// * `result_vec`: The Vec to append the resulting matrix to.
///
/// # Panics
/// * If `channel`'s width is't divisible by 8.
#[inline(always)]
fn append_matrix_at_coordinates_to_channel_matrix(
    x: usize,
    row_vectors: &[Vec<i16>],
    result_vec: &mut Vec<SMatrix<f32, 8, 8>>,
) {
    let mut iter_vector: Vec<i16> = Vec::with_capacity(64);
    for vector in row_vectors {
        let row_vec = &vector[x..x + 8];
        iter_vector.extend_from_slice(row_vec);
    }
    result_vec.push(SMatrix::<i16, 8, 8>::from_row_iterator(iter_vector).cast::<f32>());
}

fn pad_channel(channel: &mut Vec<Vec<i16>>, factor: usize) {
    let previous_len = channel.len();
    let outer_remainder = previous_len % factor;

    if outer_remainder != 0 {
        let missing_pixels = factor - outer_remainder;
        for _ in 0..missing_pixels {
            channel.push(channel[previous_len - 1].clone())
        }
    }

    let inner_remainder = channel[0].len() % factor;
    if inner_remainder != 0 {
        let missing_pixels = factor - inner_remainder;
        let desired_length = channel[0].len() + missing_pixels;

        for inner_channel in channel {
            let last_pxl_val = inner_channel.last().unwrap().clone();
            while inner_channel.len() != desired_length {
                inner_channel.push(last_pxl_val);
            }
        }
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
    pub fn pixel_at(&self, x: u16, y: u16) -> (i16, i16, i16) {
        let mut actual_y = std::cmp::max(y, 0) as usize;
        actual_y = std::cmp::min(actual_y, self.channel1.len() - 1);
        let actual_y_downsampled = if self.downsampled_vertically {
            actual_y / 2
        } else {
            actual_y
        };

        let mut actual_x = std::cmp::max(x, 0) as usize;
        actual_x = std::cmp::min(actual_x, self.channel1[actual_y].len() - 1);
        let actual_x_1 = actual_x / self.y_downsample_factor;
        let actual_x_2 = actual_x / self.cb_downsample_factor;
        let actual_x_3 = actual_x / self.cr_downsample_factor;

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
        if self.y_downsample_factor != 1
            || self.cb_downsample_factor != 1
            || self.cr_downsample_factor != 1
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
        if (product & (product - 1)) != 0 {
            panic!("One of the values is not in power of two");
        }
        let result_cb = downsample_channel(&self.channel2, a, b, c == 0);
        let cr_b = if c == 0 { b } else { c };
        let result_cr = downsample_channel(&self.channel3, a, cr_b, c == 0);

        self.channel2 = result_cb;
        self.channel3 = result_cr;

        self.cb_downsample_factor *= a / b;
        self.cr_downsample_factor *= a / cr_b;
        self.downsampled_vertically |= c == 0;
    }

    pub fn downsample_parallel(&mut self, a: usize, b: usize, c: usize) {
        if a == b && a == c && b == c {
            return;
        }
        let product = (a * b * c) as isize;
        if (product & (product - 1)) != 0 {
            panic!("One of the values is not in power of two");
        }
        let result_cb = parallel_downsample::downsample_channel(&self.channel2, a, b, c == 0);
        let cr_b = if c == 0 { b } else { c };
        let result_cr = parallel_downsample::downsample_channel(&self.channel3, a, cr_b, c == 0);

        self.channel2 = result_cb;
        self.channel3 = result_cr;

        self.cb_downsample_factor *= a / b;
        self.cr_downsample_factor *= a / cr_b;
        self.downsampled_vertically |= c == 0;
    }

    /// Get this image's data as vectors of 8x8 matrices for each of the three channels.
    /// The matrices are ordered top to bottom, then in each row left to right.
    ///
    /// # Panics
    /// * If the image's height or width cannot be divided by 8.
    pub fn to_matrices(
        &mut self,
    ) -> (
        Vec<SMatrix<f32, 8, 8>>,
        Vec<SMatrix<f32, 8, 8>>,
        Vec<SMatrix<f32, 8, 8>>,
    ) {
        self.pad_image_if_necessary();

        (
            channel_to_matrices(&self.channel1),
            channel_to_matrices(&self.channel2),
            channel_to_matrices(&self.channel3),
        )
    }

    fn pad_image_if_necessary(&mut self) {
        let y_factor : usize;
        let cb_factor : usize;
        let cr_factor : usize;

        if self.cr_downsample_factor == 2 && self.cb_downsample_factor == 2 {
            y_factor = 16;
            cb_factor = 8;
            cr_factor = 8;
        } else if self.cr_downsample_factor == 1 && self.cb_downsample_factor == 1 {
            y_factor = 8;
            cb_factor = 8;
            cr_factor = 8;
        } else {
            panic!("Unsupported downsampling!")
        }

        pad_channel(&mut self.channel1, y_factor);
        pad_channel(&mut self.channel2, cb_factor);
        pad_channel(&mut self.channel3, cr_factor);
    }

    /// Get the data of this image's first channel (Y) as a vector of 8x8 matrices.
    /// The matrices are ordered top to bottom, then in each row left to right.
    ///
    /// # Panics
    /// * If the image's height or width cannot be divided by 8.
    pub fn single_channel_to_matrices<const C: usize>(&self) -> Vec<SMatrix<f32, 8, 8>> {
        if self.channel1.len() % 8 != 0 || (self.channel1[0].len()) % 8 != 0 {
            panic!("attempted to convert image to matrices, but image dimensions are not divisible by 8 for at least one channel!");
        }
        let channel = match C {
            2 => &self.channel2,
            3 => &self.channel3,
            _ => &self.channel1,
        };
        channel_to_matrices(channel)
    }

    pub fn channel1(&self) -> &Vec<Vec<i16>> {
        &self.channel1
    }
    pub fn channel2(&self) -> &Vec<Vec<i16>> {
        &self.channel2
    }
    pub fn channel3(&self) -> &Vec<Vec<i16>> {
        &self.channel3
    }
    pub fn height(&self) -> u16 {
        self.height
    }
    pub fn width(&self) -> u16 {
        self.width
    }
    pub fn y_downsample_factor(&self) -> usize {
        self.y_downsample_factor
    }
    pub fn cb_downsample_factor(&self) -> usize {
        self.cb_downsample_factor
    }
    pub fn cr_downsample_factor(&self) -> usize {
        self.cr_downsample_factor
    }
    pub fn downsampled_vertically(&self) -> bool {
        self.downsampled_vertically
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
            y_downsample_factor: 1,
            cb_downsample_factor: 1,
            cr_downsample_factor: 1,
            downsampled_vertically: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use nalgebra::SMatrix;

    use crate::ppm_parser::read_ppm_from_file;

    use super::{convert_rgb_values_to_ycbcr, Image};

    #[test]
    fn test_downsample_image_factor_two() {
        let mut read_image = read_ppm_from_file("test/valid_test_maxVal_15.ppm");
        read_image.downsample(4, 2, 2);
        assert_eq!(
            Image {
                width: 4,
                height: 4,
                channel1: vec![
                    vec![0, 0, 0, 255],
                    vec![0, 0, 0, 0],
                    vec![0, 0, 0, 0],
                    vec![255, 0, 0, 0],
                ],
                channel2: vec![vec![0, 0], vec![127, 0], vec![0, 127], vec![0, 0]],
                channel3: vec![vec![0, 127], vec![59, 0], vec![0, 59], vec![127, 0]],
                y_downsample_factor: 1,
                cb_downsample_factor: 2,
                cr_downsample_factor: 2,
                downsampled_vertically: false,
            },
            read_image
        );
    }

    #[test]
    fn test_downsample_image_no_downsample() {
        let mut read_image = read_ppm_from_file("test/valid_test_maxVal_15.ppm");
        read_image.downsample(4, 4, 4);
        assert_eq!(
            Image {
                width: 4,
                height: 4,
                channel1: vec![
                    vec![0, 0, 0, 255],
                    vec![0, 0, 0, 0],
                    vec![0, 0, 0, 0],
                    vec![255, 0, 0, 0],
                ],
                channel2: vec![
                    vec![0, 0, 0, 0],
                    vec![0, 255, 0, 0],
                    vec![0, 0, 255, 0],
                    vec![0, 0, 0, 0],
                ],
                channel3: vec![
                    vec![0, 0, 0, 255],
                    vec![0, 119, 0, 0],
                    vec![0, 0, 119, 0],
                    vec![255, 0, 0, 0],
                ],
                y_downsample_factor: 1,
                cb_downsample_factor: 1,
                cr_downsample_factor: 1,
                downsampled_vertically: false,
            },
            read_image
        );
    }

    #[test]
    fn test_downsample_image_factor_four_and_vertical() {
        let mut read_image = read_ppm_from_file("test/valid_test_maxVal_15.ppm");
        read_image.downsample(4, 1, 0);
        assert_eq!(
            Image {
                width: 4,
                height: 4,
                channel1: vec![
                    vec![0, 0, 0, 255],
                    vec![0, 0, 0, 0],
                    vec![0, 0, 0, 0],
                    vec![255, 0, 0, 0],
                ],
                channel2: vec![vec![31], vec![31]],
                channel3: vec![vec![46], vec![46]],
                y_downsample_factor: 1,
                cb_downsample_factor: 4,
                cr_downsample_factor: 4,
                downsampled_vertically: true,
            },
            read_image
        );
    }

    #[test]
    fn test_pixel_at_in_bounds() {
        let read_image = read_ppm_from_file("test/valid_test_maxVal_15.ppm");
        let pixel = read_image.pixel_at(3, 0);
        assert_eq!((255, 0, 255), pixel);
    }

    #[test]
    fn test_pixel_at_x_out_of_bounds() {
        let read_image = read_ppm_from_file("test/valid_test_maxVal_15.ppm");
        let pixel = read_image.pixel_at(4, 0);
        assert_eq!((255, 0, 255), pixel);
    }

    #[test]
    fn test_pixel_at_y_out_of_bounds() {
        let read_image = read_ppm_from_file("test/valid_test_maxVal_15.ppm");
        let pixel = read_image.pixel_at(0, 4);
        assert_eq!((255, 0, 255), pixel);
    }

    #[test]
    fn test_pixel_at_y_and_x_out_of_bounds() {
        let read_image = read_ppm_from_file("test/valid_test_maxVal_15.ppm");
        let pixel = read_image.pixel_at(4, 4);
        assert_eq!((0, 0, 0), pixel);
    }

    #[test]
    fn test_pixel_at_in_bounds_after_downsample() {
        let mut read_image = read_ppm_from_file("test/valid_test_maxVal_15.ppm");
        read_image.downsample(4, 2, 2);
        let pixel = read_image.pixel_at(3, 0);
        assert_eq!((255, 0, 127), pixel);
    }

    #[test]
    fn test_pixel_at_x_out_of_bounds_after_downsample() {
        let mut read_image = read_ppm_from_file("test/valid_test_maxVal_15.ppm");
        read_image.downsample(4, 2, 2);
        let pixel = read_image.pixel_at(4, 0);
        assert_eq!((255, 0, 127), pixel);
    }

    #[test]
    fn test_pixel_at_y_out_of_bounds_after_vertical_downsample() {
        let mut read_image = read_ppm_from_file("test/valid_test_maxVal_15.ppm");
        read_image.downsample(4, 2, 0);
        let pixel = read_image.pixel_at(0, 4);
        assert_eq!((255, 0, 63), pixel);
    }

    #[test]
    fn test_pixel_at_y_and_x_out_of_bounds_after_downsample() {
        let mut read_image = read_ppm_from_file("test/valid_test_maxVal_15.ppm");
        read_image.downsample(4, 2, 2);
        let pixel = read_image.pixel_at(4, 4);
        assert_eq!((0, 0, 0), pixel);
    }

    #[test]
    fn test_pixel_at_y_and_x_out_of_bounds_after_vertical_downsample() {
        let mut read_image = read_ppm_from_file("test/valid_test_maxVal_15.ppm");
        read_image.downsample(4, 2, 0);
        let pixel = read_image.pixel_at(4, 4);
        assert_eq!((0, 63, 29), pixel);
    }

    fn test_convert_rgb_values_to_ycbcr_internal(start: (i16, i16, i16), target: (i16, i16, i16)) {
        let result = convert_rgb_values_to_ycbcr(start.0, start.1, start.2);
        assert_eq!(target, result);
    }

    #[test]
    fn test_convert_rgb_values_to_ycbcr_black() {
        test_convert_rgb_values_to_ycbcr_internal((0, 0, 0), (-127, 0, 0));
    }

    #[test]
    fn test_convert_rgb_values_to_ycbcr_red() {
        test_convert_rgb_values_to_ycbcr_internal((255, 0, 0), (-51, -43, 128))
    }

    #[test]
    fn test_convert_rgb_values_to_ycbcr_green() {
        test_convert_rgb_values_to_ycbcr_internal((0, 255, 0), (23, -84, -107))
    }

    #[test]
    fn test_convert_rgb_values_to_ycbcr_blue() {
        test_convert_rgb_values_to_ycbcr_internal((0, 0, 255), (-98, 128, -21))
    }

    #[test]
    fn test_convert_rgb_values_to_ycbcr_white() {
        test_convert_rgb_values_to_ycbcr_internal((255, 255, 255), (128, 0, 0))
    }

    #[test]
    fn test_convert_rgb_to_ycbcr() {
        let mut image = Image {
            height: 1,
            width: 5,
            channel1: Vec::from([Vec::from([0, 255, 0, 0, 255])]),
            channel2: Vec::from([Vec::from([0, 0, 255, 0, 255])]),
            channel3: Vec::from([Vec::from([0, 0, 0, 255, 255])]),
            ..Default::default()
        };
        image.rgb_to_ycbcr();
        let expected_image = Image {
            height: 1,
            width: 5,
            channel1: Vec::from([Vec::from([-127, -51, 23, -98, 128])]),
            channel2: Vec::from([Vec::from([0, -43, -84, 128, 0])]),
            channel3: Vec::from([Vec::from([0, 128, -107, -21, 0])]),
            ..Default::default()
        };
        assert_eq!(expected_image, image);
    }

    #[test]
    fn test_downsampling_parameters_are_power_of_two() {
        let mut image = read_ppm_from_file("test/valid_test_maxVal_15.ppm");
        image.downsample(4, 2, 2);
    }

    #[test]
    fn test_correct_scaling_not_maximal_value() {
        let image = read_ppm_from_file("test/valid_test_maxVal_15.ppm");
        let expected_image = Image {
            width: 4,
            height: 4,
            channel1: vec![
                vec![0, 0, 0, 255],
                vec![0, 0, 0, 0],
                vec![0, 0, 0, 0],
                vec![255, 0, 0, 0],
            ],
            channel2: vec![
                vec![0, 0, 0, 0],
                vec![0, 255, 0, 0],
                vec![0, 0, 255, 0],
                vec![0, 0, 0, 0],
            ],
            channel3: vec![
                vec![0, 0, 0, 255],
                vec![0, 119, 0, 0],
                vec![0, 0, 119, 0],
                vec![255, 0, 0, 0],
            ],
            y_downsample_factor: 1,
            cb_downsample_factor: 1,
            cr_downsample_factor: 1,
            downsampled_vertically: false,
        };
        assert_eq!(expected_image, image);
    }

    #[test]
    fn test_correct_scaling_maximal_value() {
        let image = read_ppm_from_file("test/valid_test_maxVal_65535.ppm");
        let expected_image = Image {
            width: 4,
            height: 4,
            channel1: vec![
                vec![0, 0, 0, 255],
                vec![0, 0, 0, 0],
                vec![0, 0, 0, 0],
                vec![255, 0, 0, 0],
            ],
            channel2: vec![
                vec![0, 0, 0, 0],
                vec![0, 255, 0, 0],
                vec![0, 0, 255, 0],
                vec![0, 0, 0, 0],
            ],
            channel3: vec![
                vec![0, 0, 0, 255],
                vec![0, 0, 0, 0], // 7 here gets lost in downscaling
                vec![0, 0, 0, 0],
                vec![255, 0, 0, 0],
            ],
            y_downsample_factor: 1,
            cb_downsample_factor: 1,
            cr_downsample_factor: 1,
            downsampled_vertically: false,
        };
        assert_eq!(expected_image, image);
    }

    #[test]
    #[should_panic]
    fn test_downsampling_a_value_not_power_of_two() {
        let mut image = read_ppm_from_file("test/valid_test_maxVal_15.ppm");
        image.downsample(5, 2, 2);
    }

    #[test]
    #[should_panic]
    fn test_downsampling_b_value_not_power_of_two() {
        let mut image = read_ppm_from_file("test/valid_test_maxVal_15.ppm");
        image.downsample(4, 3, 2);
    }

    #[test]
    #[should_panic]
    fn test_downsampling_c_value_not_power_of_two() {
        let mut image = read_ppm_from_file("test/valid_test_maxVal_15.ppm");
        image.downsample(4, 2, 3);
    }

    #[test]
    fn test_to_matrices_basic() {
        let mut image = read_ppm_from_file("test/valid_test_8x8.ppm");
        let (r, g, b) = image.to_matrices();

        let r_expected_vec = vec![
            0.0, 0.0, 0.0, 255.0, 0.0, 0.0, 0.0, 255.0, // row 1
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, // row 2
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, // row 3
            255.0, 0.0, 0.0, 0.0, 255.0, 0.0, 0.0, 0.0, // row 4
            0.0, 0.0, 0.0, 255.0, 0.0, 0.0, 0.0, 255.0, // row 5
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, // row 6
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, // row 7
            255.0, 0.0, 0.0, 0.0, 255.0, 0.0, 0.0, 0.0, // row 8
        ];
        let r_expected: Vec<SMatrix<f32, 8, 8>> = vec![SMatrix::from_iterator(r_expected_vec)];
        assert_eq!(r_expected, r);

        let g_expected_vec = vec![
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, // row 1
            0.0, 255.0, 0.0, 0.0, 0.0, 255.0, 0.0, 0.0, // row 2
            0.0, 0.0, 255.0, 0.0, 0.0, 0.0, 255.0, 0.0, // row 3
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, // row 4
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, // row 5
            0.0, 255.0, 0.0, 0.0, 0.0, 255.0, 0.0, 0.0, // row 6
            0.0, 0.0, 255.0, 0.0, 0.0, 0.0, 255.0, 0.0, // row 7
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, // row 8
        ];
        let g_expected: Vec<SMatrix<f32, 8, 8>> = vec![SMatrix::from_iterator(g_expected_vec)];
        assert_eq!(g_expected, g);

        let b_expected_vec = vec![
            0.0, 0.0, 0.0, 255.0, 0.0, 0.0, 0.0, 255.0, // row 1
            0.0, 119.0, 0.0, 0.0, 0.0, 119.0, 0.0, 0.0, // row 2
            0.0, 0.0, 119.0, 0.0, 0.0, 0.0, 119.0, 0.0, // row 3
            255.0, 0.0, 0.0, 0.0, 255.0, 0.0, 0.0, 0.0, // row 4
            0.0, 0.0, 0.0, 255.0, 0.0, 0.0, 0.0, 255.0, // row 5
            0.0, 119.0, 0.0, 0.0, 0.0, 119.0, 0.0, 0.0, // row 6
            0.0, 0.0, 119.0, 0.0, 0.0, 0.0, 119.0, 0.0, // row 7
            255.0, 0.0, 0.0, 0.0, 255.0, 0.0, 0.0, 0.0, // row 8
        ];
        let b_expected: Vec<SMatrix<f32, 8, 8>> = vec![SMatrix::from_iterator(b_expected_vec)];
        assert_eq!(b_expected, b);
    }

    #[test]
    fn test_to_matrices_downsample_and_ycbcr() {
        let mut image = read_ppm_from_file("test/valid_test_16x16.ppm");
        image.rgb_to_ycbcr();
        image.downsample(4, 2, 0);

        let (y, cb, cr) = image.to_matrices();

        let y_expected_vec = vec![
            -127.0, -127.0, -127.0, -22.0, -127.0, -127.0, -127.0, -22.0, //row 1
            -127.0, 36.0, -127.0, -127.0, -127.0, 36.0, -127.0, -127.0, //row 2
            -127.0, -127.0, 36.0, -127.0, -127.0, -127.0, 36.0, -127.0, //row 3
            -22.0, -127.0, -127.0, -127.0, -22.0, -127.0, -127.0, -127.0, //row 4
            -127.0, -127.0, -127.0, -22.0, -127.0, -127.0, -127.0, -22.0, //row 5
            -127.0, 36.0, -127.0, -127.0, -127.0, 36.0, -127.0, -127.0, //row 6
            -127.0, -127.0, 36.0, -127.0, -127.0, -127.0, 36.0, -127.0, //row 7
            -22.0, -127.0, -127.0, -127.0, -22.0, -127.0, -127.0, -127.0, // row 8
        ];
        let y_expected: Vec<SMatrix<f32, 8, 8>> = vec![
            SMatrix::from_iterator(y_expected_vec.clone()),
            SMatrix::from_iterator(y_expected_vec.clone()),
            SMatrix::from_iterator(y_expected_vec.clone()),
            SMatrix::from_iterator(y_expected_vec),
        ];
        assert_eq!(y_expected, y);

        let cb_expected_vec = vec![
            -6.0, 21.0, -6.0, 21.0, -6.0, 21.0, -6.0, 21.0, //row 1
            21.0, -6.0, 21.0, -6.0, 21.0, -6.0, 21.0, -6.0, //row 2
            -6.0, 21.0, -6.0, 21.0, -6.0, 21.0, -6.0, 21.0, //row 3
            21.0, -6.0, 21.0, -6.0, 21.0, -6.0, 21.0, -6.0, //row 4
            -6.0, 21.0, -6.0, 21.0, -6.0, 21.0, -6.0, 21.0, //row 5
            21.0, -6.0, 21.0, -6.0, 21.0, -6.0, 21.0, -6.0, //row 6
            -6.0, 21.0, -6.0, 21.0, -6.0, 21.0, -6.0, 21.0, //row 7
            21.0, -6.0, 21.0, -6.0, 21.0, -6.0, 21.0, -6.0, // row 8
        ];
        let cb_expected: Vec<SMatrix<f32, 8, 8>> = vec![SMatrix::from_iterator(cb_expected_vec)];
        assert_eq!(cb_expected, cb);

        let cr_expected_vec = vec![
            -29.0, 26.0, -29.0, 26.0, -29.0, 26.0, -29.0, 26.0, //row 1
            26.0, -29.0, 26.0, -29.0, 26.0, -29.0, 26.0, -29.0, //row 2
            -29.0, 26.0, -29.0, 26.0, -29.0, 26.0, -29.0, 26.0, //row 3
            26.0, -29.0, 26.0, -29.0, 26.0, -29.0, 26.0, -29.0, //row 4
            -29.0, 26.0, -29.0, 26.0, -29.0, 26.0, -29.0, 26.0, //row 5
            26.0, -29.0, 26.0, -29.0, 26.0, -29.0, 26.0, -29.0, //row 6
            -29.0, 26.0, -29.0, 26.0, -29.0, 26.0, -29.0, 26.0, //row 7
            26.0, -29.0, 26.0, -29.0, 26.0, -29.0, 26.0, -29.0, // row 8
        ];
        let cr_expected: Vec<SMatrix<f32, 8, 8>> = vec![SMatrix::from_iterator(cr_expected_vec)];
        assert_eq!(cr_expected, cr);
    }

    #[test]
    #[should_panic]
    #[ignore]
    fn test_to_matrices_too_small_after_downsample() {
        let mut image = read_ppm_from_file("test/valid_test_8x8.ppm");
        image.rgb_to_ycbcr();
        image.downsample(4, 2, 0);
        let _ = image.to_matrices();
    }

    #[test]
    fn test_downsample_parallel_image_factor_two() {
        let mut read_image = read_ppm_from_file("test/valid_test_maxVal_15.ppm");
        read_image.downsample_parallel(4, 2, 2);
        assert_eq!(
            Image {
                width: 4,
                height: 4,
                channel1: vec![
                    vec![0, 0, 0, 255],
                    vec![0, 0, 0, 0],
                    vec![0, 0, 0, 0],
                    vec![255, 0, 0, 0],
                ],
                channel2: vec![vec![0, 0], vec![127, 0], vec![0, 127], vec![0, 0]],
                channel3: vec![
                    vec![0, 127],
                    vec![59, 0],
                    vec![0, 59],
                    vec![127, 0],
                ],
                y_downsample_factor: 1,
                cb_downsample_factor: 2,
                cr_downsample_factor: 2,
                downsampled_vertically: false,
            },
            read_image
        );
    }

    #[test]
    fn test_downsample_parallel_image_no_downsample() {
        let mut read_image = read_ppm_from_file("test/valid_test_maxVal_15.ppm");
        read_image.downsample_parallel(4, 4, 4);
        assert_eq!(
            Image {
                width: 4,
                height: 4,
                channel1: vec![
                    vec![0, 0, 0, 255],
                    vec![0, 0, 0, 0],
                    vec![0, 0, 0, 0],
                    vec![255, 0, 0, 0],
                ],
                channel2: vec![
                    vec![0, 0, 0, 0],
                    vec![0, 255, 0, 0],
                    vec![0, 0, 255, 0],
                    vec![0, 0, 0, 0],
                ],
                channel3: vec![
                    vec![0, 0, 0, 255],
                    vec![0, 119, 0, 0],
                    vec![0, 0, 119, 0],
                    vec![255, 0, 0, 0],
                ],
                y_downsample_factor: 1,
                cb_downsample_factor: 1,
                cr_downsample_factor: 1,
                downsampled_vertically: false,
            },
            read_image
        );
    }

    #[test]
    fn test_downsample_parallel_image_factor_four_and_vertical() {
        let mut read_image = read_ppm_from_file("test/valid_test_maxVal_15.ppm");
        read_image.downsample_parallel(4, 1, 0);
        assert_eq!(
            Image {
                width: 4,
                height: 4,
                channel1: vec![
                    vec![0, 0, 0, 255],
                    vec![0, 0, 0, 0],
                    vec![0, 0, 0, 0],
                    vec![255, 0, 0, 0],
                ],
                channel2: vec![vec![31], vec![31]],
                channel3: vec![vec![46], vec![46]],
                y_downsample_factor: 1,
                cb_downsample_factor: 4,
                cr_downsample_factor: 4,
                downsampled_vertically: true,
            },
            read_image
        );
    }

    #[test]
    #[ignore]
    fn test_downsample_parallel_normal_equal() {
        let mut read_image = read_ppm_from_file("test/dwsample-ppm-640.ppm");
        let mut read_image_p = read_ppm_from_file("test/dwsample-ppm-640.ppm");
        assert_eq!(read_image, read_image_p);
        read_image.downsample(4, 1, 0);
        read_image_p.downsample_parallel(4, 1, 0);
        assert_eq!(read_image, read_image_p);
    }
}
