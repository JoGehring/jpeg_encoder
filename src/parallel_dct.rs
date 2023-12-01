use nalgebra::SMatrix;
use std::sync::mpsc::{self, Receiver};
use std::thread::{self, JoinHandle};

use crate::dct::arai_dct;
use crate::image::Image;

/// Perform the DCT on an image.
/// Performing the DCT is split up to multiple threads:
/// - At first, every channel gets the same amount of threads - if a channel is downsampled (ie has less data), it gets
/// less by its downsampling factor. As an example, if Cb is downsampled by 4 and Cr is downsampled by 2, Y gets 4 threads, Cb gets 1 thread and Cr gets 2.
/// - If an image is downsampled vertically (which is applied to Cb and Cr), Y gets twice the threads as it has twice the data.
/// - While the CPU has more than twice the now distributed amount of threads available, double the threads for each channel. So if you have 6 threads (as you would
///     if you downsampled 4-2-0) and the CPU supports 12 or more threads, double them. If the CPU supports 24 or more threads, quadruple them.
/// 
/// # Arguments
/// * `image`: The image to calculate the DCT for.
pub fn dct(
    image: &Image,
) -> (
    Vec<SMatrix<i32, 8, 8>>,
    Vec<SMatrix<i32, 8, 8>>,
    Vec<SMatrix<i32, 8, 8>>,
) {
    let (y_matrices, cb_matrices, cr_matrices) = image.to_matrices();
    let (y_threads, cb_threads, cr_threads) = calculate_number_of_threads(image);

    let y_capacity = y_matrices.len();
    let cb_capacity = cb_matrices.len();
    let cr_capacity = cr_matrices.len();

    let (y_handles, y_receivers) = spawn_threads_for_channel(y_matrices, y_threads);
    let (cb_handles, cb_receivers) = spawn_threads_for_channel(cb_matrices, cb_threads);
    let (cr_handles, cr_receivers) = spawn_threads_for_channel(cr_matrices, cr_threads);

    let y_result = join_and_receive_threads_for_channel(y_handles, y_receivers, y_capacity);
    let cb_result = join_and_receive_threads_for_channel(cb_handles, cb_receivers, cb_capacity);
    let cr_result = join_and_receive_threads_for_channel(cr_handles, cr_receivers, cr_capacity);

    (y_result, cb_result, cr_result)
}

/// Calculate the number of threads for each channel.
/// - At first, every channel gets the same amount of threads - if a channel is downsampled (ie has less data), it gets
/// less by its downsampling factor. As an example, if Cb is downsampled by 4 and Cr is downsampled by 2, Y gets 4 threads, Cb gets 1 thread and Cr gets 2.
/// - If an image is downsampled vertically (which is applied to Cb and Cr), Y gets twice the threads as it has twice the data.
/// - While the CPU has more than twice the now distributed amount of threads available, double the threads for each channel. So if you have 6 threads (as you would
///     if you downsampled 4-2-0) and the CPU supports 12 or more threads, double them. If the CPU supports 24 or more threads, quadruple them.
/// 
/// # Arguments
/// * `image`: The image to calculate the DCT for.
fn calculate_number_of_threads(image: &Image) -> (usize, usize, usize) {
    let max_downsample_factor =
        std::cmp::max(image.cb_downsample_factor(), image.cr_downsample_factor());
    let mut y_threads = max_downsample_factor;
    let mut cb_threads = max_downsample_factor / image.cb_downsample_factor();
    let mut cr_threads = max_downsample_factor / image.cr_downsample_factor();
    if image.downsampled_vertically() {
        y_threads *= 2;
    }

    let available_threads = thread::available_parallelism().unwrap().get();
    let factor = std::cmp::max(1, available_threads / (y_threads + cb_threads + cr_threads));
    y_threads *= factor;
    cb_threads *= factor;
    cr_threads *= factor;

    (y_threads, cb_threads, cr_threads)
}

/// Spawn the worker threads for each channel.
/// The channel data is split up into chunks of equal size,
/// each of which is then passed into its own thread.
/// 
/// # Arguments
/// * `channel`: The channel of data to calculate the DCT on.
/// * `thread_count`: The number of threads this channel gets.
fn spawn_threads_for_channel(
    channel: Vec<SMatrix<u16, 8, 8>>,
    thread_count: usize,
) -> (Vec<JoinHandle<()>>, Vec<Receiver<Vec<SMatrix<i32, 8, 8>>>>) {
    let data_vecs: Vec<&[SMatrix<u16, 8, 8>]> = channel.chunks(thread_count).collect();
    let mut handles: Vec<JoinHandle<()>> = Vec::with_capacity(data_vecs.len());
    let mut receivers: Vec<Receiver<Vec<SMatrix<i32, 8, 8>>>> = Vec::with_capacity(data_vecs.len());

    for data in data_vecs {
        let (tx, rx) = mpsc::channel();
        // slow copy because directly using `data` leads to borrow issues. maybe fixable with lifetimes?
        let data_vec = data.to_vec();

        let handle = thread::spawn(move || {
            let mut result: Vec<SMatrix<i32, 8, 8>> = Vec::with_capacity(data_vec.len());
            for matrix in data_vec {
                result.push(arai_dct(&matrix))
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
    receivers: Vec<Receiver<Vec<SMatrix<i32, 8, 8>>>>,
    capacity: usize,
) -> Vec<SMatrix<i32, 8, 8>> {
    let mut result: Vec<SMatrix<i32, 8, 8>> = Vec::with_capacity(capacity);
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
    use nalgebra::SMatrix;

    use crate::read_ppm_from_file;
    #[test]
    fn test_dct_parallel_simple_image() {
        let image = read_ppm_from_file("test/valid_test_8x8.ppm");

        let (y, cb, cr) = crate::parallel_dct::dct(&image);

        let y_expected_vec: Vec<i32> = vec![
            65535, 0, 0, 0, 65535, 0, 0, 0, // row 1
            0, -20228, 0, -35709, 0, 7103, 0, -30273, // row 2
            0, 0, 0, 0, 0, 0, 0, 0, 0, // row 3
            -35710, 0, -63041, 0, 12540, 0, -53444, 65535, // row 4
            0, 0, 0, 65535, 0, 0, 0, 0, // row 5
            7103, 0, 12540, 0, -2494, 0, 10631, 0, // row 6
            0, 0, 0, 0, 0, 0, 0, 0, // row 7
            -30274, 0, -53444, 0, 10631, 0, -45308, // row 8
        ];
        let y_expected: Vec<SMatrix<i32, 8, 8>> = vec![SMatrix::from_iterator(y_expected_vec)];

        let cb_expected_vec: Vec<i32> = vec![
            65535, 0, 0, 0, -65535, 0, 0, 0, // row 1
            0, 2494, 0, 7103, 0, -10631, 0, -12540, // row 2
            0, 0, 0, 0, 0, 0, 0, 0, // row 3
            0, 7103, 0, 20228, 0, -30273, 0, -35709, // row 4
            -65535, 0, 0, 0, 65535, 0, 0, 0, 0, // row 5
            -10631, 0, -30274, 0, 45308, 0, 53444, 0, // row 6
            0, 0, 0, 0, 0, 0, 0, 0, // row 7
            -12540, 0, -35710, 0, 53444, 0, 63041, // row 8
        ];
        let cb_expected: Vec<SMatrix<i32, 8, 8>> = vec![SMatrix::from_iterator(cb_expected_vec)];

        let cr_expected_vec: Vec<i32> = vec![
            96117, 0, 0, 0, 34952, 0, 0, 0, // row 1
            0, -19064, 0, -32394, 0, 2142, 0, -36125, // row 2
            0, 0, 0, 0, 0, 0, 0, 0, // row 3
            0, -32395, 0, -53602, 0, -1587, 0, -70107, // row 4
            34952, 0, 0, 0, 96117, 0, 0, 0, // row 5
            0, 2143, 0, -1587, 0, 18649, 0, 35571, // row 6
            0, 0, 0, 0, 0, 0, 0, 0, // row 7
            0, -36125, 0, -70109, 0, 35571, 0, -15889, // row 8
        ];
        let cr_expected: Vec<SMatrix<i32, 8, 8>> = vec![SMatrix::from_iterator(cr_expected_vec)];

        assert_eq!(y_expected, y);
        assert_eq!(cb_expected, cb);
        assert_eq!(cr_expected, cr);
    }
}
