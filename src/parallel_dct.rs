use std::slice::ChunksMut;
use std::thread;

use nalgebra::SMatrix;

use crate::dct::{arai_dct, direct_dct, matrix_dct, DCTMode};
use crate::image::Image;

/// Perform the DCT on an image.
/// The DCT is performed for each channel in sequence.
/// DCT on a channel is parallelised with as many threads as the system has logical CPUs.
///
/// # Arguments
/// * `image`: The image to calculate the DCT for.
pub fn dct(
    image: &Image,
    mode: &DCTMode, // perhaps make this a generic? does that help at compile time?
) -> (
    Vec<SMatrix<f32, 8, 8>>,
    Vec<SMatrix<f32, 8, 8>>,
    Vec<SMatrix<f32, 8, 8>>,
) {
    let function = match mode {
        DCTMode::Direct => direct_dct,
        DCTMode::Matrix => matrix_dct,
        DCTMode::Arai => arai_dct,
    };

    let (mut y_matrices, mut cb_matrices, mut cr_matrices) = image.to_matrices();

    dct_channel(&mut y_matrices, &function);
    dct_channel(&mut cb_matrices, &function);
    dct_channel(&mut cr_matrices, &function);
    (y_matrices, cb_matrices, cr_matrices)
}

/// Perform the DCT on only the image's 'Y' channel.
/// The DCT on a channel is parallelised with as many threads as the system has logical CPUs.
///
/// # Arguments
/// * `image`: The image to calculate the DCT for.
pub fn dct_single_channel(image: &Image, mode: &DCTMode) -> Vec<SMatrix<f32, 8, 8>> {
    let function = match mode {
        DCTMode::Direct => crate::dct::direct_dct,
        DCTMode::Matrix => crate::dct::matrix_dct,
        DCTMode::Arai => crate::dct::arai_dct,
    };
    let mut y_matrices = image.single_channel_to_matrices::<1>();

    dct_channel(&mut y_matrices, &function);
    y_matrices
}

/// Perform the DCT on a matrix vector representation of an image.
/// The DCT on a channel is parallelised with as many threads as the system has logical CPUs.
///
/// # Arguments
/// * `image`: The image to calculate the DCT for.
pub fn dct_matrix_vector(matrices: &mut Vec<SMatrix<f32, 8, 8>>, mode: &DCTMode) {
    let function = match mode {
        DCTMode::Direct => direct_dct,
        DCTMode::Matrix => matrix_dct,
        DCTMode::Arai => arai_dct,
    };

    dct_channel(matrices, &function);
}

/// process the channel.
/// The channel data is split up into chunks of equal size,
/// each of which is then passed into its own thread.
/// This uses as many threads as the system has logical CPUs.
///
/// # Arguments
/// * `channel`: The channel of data to calculate the DCT on.
/// * `function`: The DCT function to use.
fn dct_channel(channel: &mut Vec<SMatrix<f32, 8, 8>>, function: &fn(&mut SMatrix<f32, 8, 8>)) {
    let thread_count = thread::available_parallelism().unwrap().get();
    let chunk_size = (channel.len() / thread_count) + 1;
    let chunks: ChunksMut<SMatrix<f32, 8, 8>> = channel.chunks_mut(chunk_size);
    thread::scope(|s| {
        let mut handles = Vec::with_capacity(chunks.len());
        for chunk in chunks {
            handles.push(s.spawn(move || {
                for mut matrix in chunk {
                    function(&mut matrix);
                }
            }));
        }
        for handle in handles {
            handle.join().unwrap();
        }
    });
}

#[cfg(test)]
mod tests {
    use approx::assert_abs_diff_eq;
    use nalgebra::SMatrix;

    use crate::ppm_parser::read_ppm_from_file;

    #[test]
    fn test_dct_parallel_simple_image() {
        let image = read_ppm_from_file("test/valid_test_8x8.ppm");

        let (y, cb, cr) = crate::parallel_dct::dct(&image, &crate::dct::DCTMode::Arai);

        let y_expected_vec: Vec<f32> = vec![
            65534.996, 0.0, 0.0, 0.0, 65534.996, 0.0, 0.0, 0.0, // row 1
            0.0, -20227.922, 0.0, -35709.7, 0.0, 7103.1016, 0.0, -30273.236, // row 2
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, // row 3
            -35709.7, 0.0, -63040.715, 0.0, 12539.579, 0.0, -53443.37, 65534.996, // row 4
            0.0, 0.0, 0.0, 65534.996, 0.0, 0.0, 0.0, 0.0, // row 5
            7103.1035, 0.0, 12539.577, 0.0, -2494.2773, 0.0, 10630.548, 0.0, // row 6
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, // row 7
            -30273.24, 0.0, -53443.367, 0.0, 10630.547, 0.0, -45307.133, // row 8
        ];
        let y_expected: Vec<SMatrix<f32, 8, 8>> = vec![SMatrix::from_iterator(y_expected_vec)];

        let cb_expected_vec: Vec<f32> = vec![
            65534.996, 0.0, 0.0, 0.0, -65534.996, 0.0, 0.0, 0.0, // row 1
            0.0, 2494.2776, 0.0, 7103.0996, 0.0, -10630.543, 0.0, -12539.582, // row 2
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, // row 3
            0.0, 7103.101, 0.0, 20227.92, 0.0, -30273.223, 0.0, -35709.715, // row 4
            -65534.996, 0.0, 0.0, 0.0, 65534.996, 0.0, 0.0, 0.0, 0.0, // row 5
            -10630.543, 0.0, -30273.225, 0.0, 45307.086, 0.0, 53443.37, 0.0, // row 6
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, // row 7
            -12539.585, 0.0, -35709.707, 0.0, 53443.367, 0.0, 63040.758, // row 8
        ];
        let cb_expected: Vec<SMatrix<f32, 8, 8>> = vec![SMatrix::from_iterator(cb_expected_vec)];

        let cr_expected_vec: Vec<f32> = vec![
            96118.01,
            0.00069053395,
            0.0,
            0.0,
            34951.996,
            0.0,
            0.0,
            0.0, // row 1
            0.0,
            -19063.924,
            0.0,
            -32394.918,
            0.0,
            2142.1814,
            0.0,
            -36125.047, // row 2
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0, // row 3
            0.0,
            -32394.922,
            0.0,
            -53601.016,
            0.0,
            -1587.9264,
            0.0,
            -70107.9, // row 4
            34951.996,
            0.0,
            0.0,
            0.0,
            96118.01,
            0.0,
            0.0,
            0.0, // row 5
            0.0,
            2142.1826,
            0.0,
            -1587.9272,
            0.0,
            18649.031,
            0.0,
            35570.785, // row 6
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0, // row 7
            0.0,
            -36125.05,
            0.0,
            -70107.91,
            0.0,
            35570.79,
            0.0,
            -15888.09, // row 8
        ];
        let cr_expected: Vec<SMatrix<f32, 8, 8>> = vec![SMatrix::from_iterator(cr_expected_vec)];

        for index in 0..y_expected.len() {
            for i in 0..8 {
                for j in 0..8 {
                    assert_abs_diff_eq!(
                        y_expected[index][(i, j)],
                        y[index][(i, j)],
                        epsilon = 0.01
                    );
                    assert_abs_diff_eq!(
                        cb_expected[index][(i, j)],
                        cb[index][(i, j)],
                        epsilon = 0.01
                    );
                    assert_abs_diff_eq!(
                        cr_expected[index][(i, j)],
                        cr[index][(i, j)],
                        epsilon = 0.01
                    );
                }
            }
        }
    }

    #[test]
    fn test_single_channel_simple_image() {
        let image = read_ppm_from_file("test/valid_test_8x8.ppm");

        let y = crate::parallel_dct::dct_single_channel(&image, &crate::dct::DCTMode::Arai);

        let y_expected_vec: Vec<f32> = vec![
            65534.996, 0.0, 0.0, 0.0, 65534.996, 0.0, 0.0, 0.0, // row 1
            0.0, -20227.922, 0.0, -35709.7, 0.0, 7103.1016, 0.0, -30273.236, // row 2
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, // row 3
            -35709.7, 0.0, -63040.715, 0.0, 12539.579, 0.0, -53443.37, 65534.996, // row 4
            0.0, 0.0, 0.0, 65534.996, 0.0, 0.0, 0.0, 0.0, // row 5
            7103.1035, 0.0, 12539.577, 0.0, -2494.2773, 0.0, 10630.548, 0.0, // row 6
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, // row 7
            -30273.24, 0.0, -53443.367, 0.0, 10630.547, 0.0, -45307.133, // row 8
        ];
        let y_expected: Vec<SMatrix<f32, 8, 8>> = vec![SMatrix::from_iterator(y_expected_vec)];
        for index in 0..y_expected.len() {
            for i in 0..8 {
                for j in 0..8 {
                    assert_abs_diff_eq!(
                        y_expected[index][(i, j)],
                        y[index][(i, j)],
                        epsilon = 0.01
                    );
                }
            }
        }
    }
}
