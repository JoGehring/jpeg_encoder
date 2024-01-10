use std::slice::ChunksMut;

use nalgebra::SMatrix;
use scoped_threadpool::Pool;

use crate::dct::{arai_dct, DCTMode, direct_dct, matrix_dct};
use crate::image::Image;
use crate::utils::THREAD_COUNT;

/// Perform the DCT on an image.
/// The DCT is performed for each channel in sequence.
/// DCT on a channel is parallelised with as many threads as the system has logical CPUs.
///
/// # Arguments
/// * `image`: The image to calculate the DCT for.
pub fn dct(
    image: &Image,
    mode: &DCTMode, // perhaps make this a generic? does that help at compile time?
    pool: &mut Pool,
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

    dct_channel(&mut y_matrices, &function, pool);
    dct_channel(&mut cb_matrices, &function, pool);
    dct_channel(&mut cr_matrices, &function, pool);
    (y_matrices, cb_matrices, cr_matrices)
}

/// Perform the DCT on only the image's 'Y' channel.
/// The DCT on a channel is parallelised with as many threads as the system has logical CPUs.
///
/// # Arguments
/// * `image`: The image to calculate the DCT for.
pub fn dct_single_channel(
    image: &Image,
    mode: &DCTMode,
    pool: &mut Pool,
) -> Vec<SMatrix<f32, 8, 8>> {
    let function = match mode {
        DCTMode::Direct => direct_dct,
        DCTMode::Matrix => matrix_dct,
        DCTMode::Arai => arai_dct,
    };
    let mut y_matrices = image.single_channel_to_matrices::<1>();

    dct_channel(&mut y_matrices, &function, pool);
    y_matrices
}

/// Perform the DCT on a matrix vector representation of an image.
/// The DCT on a channel is parallelised with as many threads as the system has logical CPUs.
///
/// # Arguments
/// * `image`: The image to calculate the DCT for.
pub fn dct_matrix_vector(matrices: &mut Vec<SMatrix<f32, 8, 8>>, mode: &DCTMode, pool: &mut Pool) {
    let function = match mode {
        DCTMode::Direct => direct_dct,
        DCTMode::Matrix => matrix_dct,
        DCTMode::Arai => arai_dct,
    };

    dct_channel(matrices, &function, pool);
}

/// process the channel.
/// The channel data is split up into chunks of equal size,
/// each of which is then passed into its own thread.
/// This uses as many threads as the system has logical CPUs.
///
/// # Arguments
/// * `channel`: The channel of data to calculate the DCT on.
/// * `function`: The DCT function to use.
fn dct_channel(
    channel: &mut Vec<SMatrix<f32, 8, 8>>,
    function: &fn(&mut SMatrix<f32, 8, 8>),
    pool: &mut Pool,
) {
    let chunk_size = (channel.len() / *THREAD_COUNT) + 1;
    let chunks: ChunksMut<SMatrix<f32, 8, 8>> = channel.chunks_mut(chunk_size);
    pool.scoped(|s| {
        for chunk in chunks {
            s.execute(move || {
                for matrix in chunk {
                    function(matrix);
                }
            });
        }
    });
}

#[cfg(test)]
mod tests {
    use std::thread::available_parallelism;

    use approx::assert_abs_diff_eq;
    use nalgebra::SMatrix;
    use scoped_threadpool::Pool;

    use crate::ppm_parser::read_ppm_from_file;

    use super::dct;

    fn get_pool() -> Pool {
        let thread_count = available_parallelism().unwrap().get();
        return Pool::new(thread_count as u32);
    }

    #[test]
    fn test_dct_parallel_simple_image() {
        let mut pool = get_pool();

        let image = read_ppm_from_file("test/valid_test_8x8.ppm");

        let (y, cb, cr) = dct(&image, &crate::dct::DCTMode::Arai, &mut pool);

        let y_expected_vec: Vec<f32> = vec![
            255.0, 0.0, 0.0, 0.0, 255.0, 0.0, 0.0, 0.0, // row 1
            0.0, -78.70786, 0.0, -138.94827, 0.0, 27.6385256, 0.0, -117.79471, // row 2
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, // row 3
            -138.94827, 0.0, -245.29465, 0.0, 48.792145, 0.0, -207.9509, 255.0, // row 4
            0.0, 0.0, 0.0, 255.0, 0.0, 0.0, 0.0, 0.0, // row 5
            27.638529, 0.0, 48.79214, 0.0, -9.705362, 0.0, 41.364006, 0.0, // row 6
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, // row 7
            -117.79469, 0.0, -207.95085, 0.0, 41.363987, 0.0, -176.29231, // row 8
        ];
        let y_expected: Vec<SMatrix<f32, 8, 8>> = vec![SMatrix::from_iterator(y_expected_vec)];

        let cb_expected_vec: Vec<f32> = vec![
            255.0,
            0.0,
            0.0,
            0.0,
            -255.0,
            0.0,
            0.0,
            0.0, // row 1
            0.0,
            9.705359,
            0.0,
            27.638,
            0.0,
            -41.36398,
            0.0,
            -48.792156, // row 2
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0, // row 3
            0.0,
            27.638525,
            0.0,
            78.707855,
            0.0,
            -117.79465,
            0.0,
            -138.9482715, // row 4
            -255.0,
            0.0,
            0.0,
            0.0,
            255.0,
            0.0,
            0.0,
            0.0,
            0.0, // row 5
            -41.36398,
            0.0,
            -117.79465,
            0.0,
            176.29218,
            0.0,
            207.9509,
            0.0, // row 6
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0, // row 7
            -48.79216,
            0.0,
            -138.9482707,
            0.0,
            207.95087,
            0.0,
            245.29485, // row 8
        ];
        let cb_expected: Vec<SMatrix<f32, 8, 8>> = vec![SMatrix::from_iterator(cb_expected_vec)];

        let cr_expected_vec: Vec<f32> = vec![
            374.0,
            0.00069053395,
            0.0,
            0.0,
            136.0,
            0.0,
            0.0,
            0.0, // row 1
            0.0,
            -74.178696,
            0.0,
            -126.05028,
            0.0,
            8.335341,
            0.0,
            -140.56438, // row 2
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0, // row 3
            0.0,
            -126.05029,
            0.0,
            -208.56433,
            0.0,
            -6.1786857,
            0.0,
            -272.7934, // row 4
            136.0,
            0.0,
            0.0,
            0.0,
            374.0,
            0.0,
            0.0,
            0.0, // row 5
            0.0,
            8.335339,
            0.0,
            -6.1786933,
            0.0,
            72.56431,
            0.0,
            138.40773, // row 6
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0, // row 7
            0.0,
            -140.56438,
            0.0,
            -272.79337,
            0.0,
            138.40771,
            0.0,
            -61.821407, // row 8
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
        let mut pool = get_pool();

        let image = read_ppm_from_file("test/valid_test_8x8.ppm");

        let y =
            crate::parallel_dct::dct_single_channel(&image, &crate::dct::DCTMode::Arai, &mut pool);

        let y_expected_vec: Vec<f32> = vec![
            255.0, 0.0, 0.0, 0.0, 255.0, 0.0, 0.0, 0.0, // row 1
            0.0, -78.70786, 0.0, -138.94827, 0.0, 27.638525, 0.0, -117.79471, // row 2
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, // row 3
            -138.94827, 0.0, -245.29465, 0.0, 48.792145, 0.0, -207.9509, 255.0, // row 4
            0.0, 0.0, 0.0, 255.0, 0.0, 0.0, 0.0, 0.0, // row 5
            27.638529, 0.0, 48.79214, 0.0, -9.705362, 0.0, 41.364006, 0.0, // row 6
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, // row 7
            -117.79469, 0.0, -207.95085, 0.0, 41.363987, 0.0, -176.29231, // row 8
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
