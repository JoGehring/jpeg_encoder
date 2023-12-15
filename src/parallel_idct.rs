use nalgebra::SMatrix;
use std::sync::mpsc::{self, Receiver};
use std::thread::{self, JoinHandle};

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
            65534.996, 0.0, 0.0, 0.0, 65534.996, 0.0, 0.0, 0.0, // row 1
            0.0, -20227.922, 0.0, -35709.7, 0.0, 7103.1016, 0.0, -30273.236, // row 2
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, // row 3
            -35709.7, 0.0, -63040.715, 0.0, 12539.579, 0.0, -53443.37, 65534.996, // row 4
            0.0, 0.0, 0.0, 65534.996, 0.0, 0.0, 0.0, 0.0, // row 5
            7103.1035, 0.0, 12539.577, 0.0, -2494.2773, 0.0, 10630.548, 0.0, // row 6
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, // row 7
            -30273.24, 0.0, -53443.367, 0.0, 10630.547, 0.0, -45307.133, // row 8
        ];
        let y_dct: Vec<SMatrix<f32, 8, 8>> = vec![SMatrix::from_iterator(y_dct_vec)];

        let cb_dct_vec: Vec<f32> = vec![
            65534.996, 0.0, 0.0, 0.0, -65534.996, 0.0, 0.0, 0.0, // row 1
            0.0, 2494.2776, 0.0, 7103.0996, 0.0, -10630.543, 0.0, -12539.582, // row 2
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, // row 3
            0.0, 7103.101, 0.0, 20227.92, 0.0, -30273.223, 0.0, -35709.715, // row 4
            -65534.996, 0.0, 0.0, 0.0, 65534.996, 0.0, 0.0, 0.0, 0.0, // row 5
            -10630.543, 0.0, -30273.225, 0.0, 45307.086, 0.0, 53443.37, 0.0, // row 6
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, // row 7
            -12539.585, 0.0, -35709.707, 0.0, 53443.367, 0.0, 63040.758, // row 8
        ];
        let cb_dct: Vec<SMatrix<f32, 8, 8>> = vec![SMatrix::from_iterator(cb_dct_vec)];

        let cr_dct_vec: Vec<f32> = vec![
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
        let cr_dct: Vec<SMatrix<f32, 8, 8>> = vec![SMatrix::from_iterator(cr_dct_vec)];

        let (y, cb, cr) = idct(&y_dct, &cb_dct, &cr_dct);
        for index in 0..cb_dct.len() {
            for i in 0..8 {
                for j in 0..8 {
                    assert_abs_diff_eq!(y_expected[index][(i, j)], y[index][(i, j)], epsilon = 1.0);
                    assert_abs_diff_eq!(cb_expected[index][(i, j)], cb[index][(i, j)], epsilon = 1.0);
                    assert_abs_diff_eq!(cr_expected[index][(i, j)], cr[index][(i, j)], epsilon = 1.0);
                }
            }
        }
    }
}
