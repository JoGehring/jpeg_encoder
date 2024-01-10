use std::sync::mpsc::{self, Receiver};
use std::thread::{self, JoinHandle};

use nalgebra::SMatrix;

use crate::dct::inverse_dct;
use crate::utils::THREAD_COUNT;

/// Perform the inverse DCT on an image.
/// The inverse DCT is performed for each channel in sequence.
/// DCT on a channel is parallelised with as many threads as the system has logical CPUs.
///
/// # Arguments
pub fn idct(
    y_matrices: &Vec<SMatrix<f32, 8, 8>>,
    cb_matrices: &Vec<SMatrix<f32, 8, 8>>,
    cr_matrices: &Vec<SMatrix<f32, 8, 8>>,
) -> (
    Vec<SMatrix<f32, 8, 8>>,
    Vec<SMatrix<f32, 8, 8>>,
    Vec<SMatrix<f32, 8, 8>>,
) {
    //each matrix holds 64 values
    let y_capacity = y_matrices.len();
    let cb_capacity = cb_matrices.len();
    let cr_capacity = cr_matrices.len();

    let (y_handles, y_receivers) = spawn_threads_for_channel(y_matrices);
    let y_result = join_and_receive_threads_for_channel(y_handles, y_receivers, y_capacity);

    let (cb_handles, cb_receivers) = spawn_threads_for_channel(cb_matrices);
    let cb_result = join_and_receive_threads_for_channel(cb_handles, cb_receivers, cb_capacity);

    let (cr_handles, cr_receivers) = spawn_threads_for_channel(cr_matrices);
    let cr_result = join_and_receive_threads_for_channel(cr_handles, cr_receivers, cr_capacity);

    (y_result, cb_result, cr_result)
}

/// Spawn the worker threads for each channel.
/// The channel data is split up into chunks of equal size,
/// each of which is then passed into its own thread.
///
/// # Arguments
/// * `channel`: The channel of data to calculate the DCT on.
/// * `thread_count`: The number of threads this channel gets.
fn spawn_threads_for_channel(
    channel: &Vec<SMatrix<f32, 8, 8>>,
) -> (Vec<JoinHandle<()>>, Vec<Receiver<Vec<SMatrix<f32, 8, 8>>>>) {
    // + 1 to avoid creating a new chunk with just the last element
    let chunk_size = (channel.len() / *THREAD_COUNT) + 1;
    let data_vecs: std::slice::Chunks<'_, SMatrix<f32, 8, 8>> = channel.chunks(chunk_size);
    let mut handles: Vec<JoinHandle<()>> = Vec::with_capacity(*THREAD_COUNT);
    let mut receivers: Vec<Receiver<Vec<SMatrix<f32, 8, 8>>>> = Vec::with_capacity(*THREAD_COUNT);

    for data in data_vecs {
        let (tx, rx) = mpsc::channel();
        // slow copy because directly using `data` leads to borrow issues. maybe fixable with lifetimes?
        let data_vec = data.to_vec();

        let handle = thread::spawn(move || {
            let mut result: Vec<SMatrix<f32, 8, 8>> = Vec::with_capacity(data_vec.len());
            for matrix in data_vec {
                result.push(inverse_dct(&matrix))
            }
            tx.send(result).unwrap()
        });

        handles.push(handle);
        receivers.push(rx);
    }

    (handles, receivers)
}

/// Join and receive worker threads for this channel,
/// then combine their resulting data into a single Vec.
///
/// # Arguments
/// * `handles`: The thread handles.
/// * `receivers`: The message receivers for each thread.
/// * `capacity`: The amount of matrices in the result. Used to avoid having to reallocate.
fn join_and_receive_threads_for_channel(
    handles: Vec<JoinHandle<()>>,
    receivers: Vec<Receiver<Vec<SMatrix<f32, 8, 8>>>>,
    capacity: usize,
) -> Vec<SMatrix<f32, 8, 8>> {
    let mut result: Vec<SMatrix<f32, 8, 8>> = Vec::with_capacity(capacity);
    for handle in handles {
        handle.join().unwrap();
    }
    for receiver in receivers {
        result.extend(receiver.recv().unwrap());
    }
    result
}

#[cfg(test)]
mod tests {
    use approx::assert_abs_diff_eq;
    use nalgebra::SMatrix;

    use crate::ppm_parser::read_ppm_from_file;

    use super::idct;

    #[test]
    fn test_idct_parallel_simple_image() {
        let image = read_ppm_from_file("test/valid_test_8x8.ppm");
        let (y_expected, cb_expected, cr_expected) = image.to_matrices();

        let y_dct_vec: Vec<f32> = vec![
            255.0, 0.0, 0.0, 0.0, 255.0, 0.0, 0.0, 0.0, // row 1
            0.0, -78.70786, 0.0, -138.94827, 0.0, 27.6385256, 0.0, -117.79471, // row 2
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, // row 3
            -138.94827, 0.0, -245.29465, 0.0, 48.792145, 0.0, -207.9509, 255.0, // row 4
            0.0, 0.0, 0.0, 255.0, 0.0, 0.0, 0.0, 0.0, // row 5
            27.638529, 0.0, 48.79214, 0.0, -9.705362, 0.0, 41.364006, 0.0, // row 6
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, // row 7
            -117.79469, 0.0, -207.95085, 0.0, 41.363987, 0.0, -176.29231, // row 8
        ];
        let y_dct: Vec<SMatrix<f32, 8, 8>> = vec![SMatrix::from_iterator(y_dct_vec)];

        let cb_dct_vec: Vec<f32> = vec![
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
        let cb_dct: Vec<SMatrix<f32, 8, 8>> = vec![SMatrix::from_iterator(cb_dct_vec)];

        let cr_dct_vec: Vec<f32> = vec![
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
        let cr_dct: Vec<SMatrix<f32, 8, 8>> = vec![SMatrix::from_iterator(cr_dct_vec)];

        let (y, cb, cr) = idct(&y_dct, &cb_dct, &cr_dct);
        for index in 0..cb_dct.len() {
            for i in 0..8 {
                for j in 0..8 {
                    assert_abs_diff_eq!(y_expected[index][(i, j)], y[index][(i, j)], epsilon = 1.0);
                    assert_abs_diff_eq!(
                        cb_expected[index][(i, j)],
                        cb[index][(i, j)],
                        epsilon = 1.0
                    );
                    assert_abs_diff_eq!(
                        cr_expected[index][(i, j)],
                        cr[index][(i, j)],
                        epsilon = 1.0
                    );
                }
            }
        }
    }
}
