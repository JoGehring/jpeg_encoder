use nalgebra::SMatrix;
use std::sync::mpsc::{self, Receiver};
use std::thread::{self, JoinHandle};

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
    Vec<SMatrix<i32, 8, 8>>,
    Vec<SMatrix<i32, 8, 8>>,
    Vec<SMatrix<i32, 8, 8>>,
) {
    let function = match mode {
        DCTMode::Direct => direct_dct,
        DCTMode::Matrix => matrix_dct,
        DCTMode::Arai => arai_dct,
    };

    let (y_matrices, cb_matrices, cr_matrices) = image.to_matrices();

    let y_capacity = y_matrices.len();
    let cb_capacity = cb_matrices.len();
    let cr_capacity = cr_matrices.len();

    let (y_handles, y_receivers) = spawn_threads_for_channel(y_matrices, function);
    let y_result = join_and_receive_threads_for_channel(y_handles, y_receivers, y_capacity);

    let (cb_handles, cb_receivers) = spawn_threads_for_channel(cb_matrices, function);
    let cb_result = join_and_receive_threads_for_channel(cb_handles, cb_receivers, cb_capacity);

    let (cr_handles, cr_receivers) = spawn_threads_for_channel(cr_matrices, function);
    let cr_result = join_and_receive_threads_for_channel(cr_handles, cr_receivers, cr_capacity);

    (y_result, cb_result, cr_result)
}

/// Perform the DCT on only the image's 'Y' channel.
/// The DCT on a channel is parallelised with as many threads as the system has logical CPUs.
///
/// # Arguments
/// * `image`: The image to calculate the DCT for.
pub fn dct_single_channel(image: &Image, mode: &DCTMode) -> Vec<SMatrix<i32, 8, 8>> {
    let function = match mode {
        DCTMode::Direct => crate::dct::direct_dct,
        DCTMode::Matrix => crate::dct::matrix_dct,
        DCTMode::Arai => crate::dct::arai_dct,
    };
    let y_matrices = image.single_channel_to_matrices();

    let y_capacity = y_matrices.len();

    let (y_handles, y_receivers) = spawn_threads_for_channel(y_matrices, function);
    let y_result = join_and_receive_threads_for_channel(y_handles, y_receivers, y_capacity);

    y_result
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
    function: fn(&SMatrix<u16, 8, 8>) -> SMatrix<i32, 8, 8>,
) -> (Vec<JoinHandle<()>>, Vec<Receiver<Vec<SMatrix<i32, 8, 8>>>>) {
    let thread_count = std::thread::available_parallelism().unwrap().get();
    // + 1 to avoid creating a new chunk with just the last element
    let chunk_size = (channel.len() / thread_count) + 1;
    let data_vecs: std::slice::Chunks<'_, SMatrix<u16, 8, 8>> = channel.chunks(chunk_size);
    let mut handles: Vec<JoinHandle<()>> = Vec::with_capacity(thread_count);
    let mut receivers: Vec<Receiver<Vec<SMatrix<i32, 8, 8>>>> = Vec::with_capacity(thread_count);

    for data in data_vecs {
        let (tx, rx) = mpsc::channel();
        // slow copy because directly using `data` leads to borrow issues. maybe fixable with lifetimes?
        let data_vec = data.to_vec();

        let handle = thread::spawn(move || {
            let mut result: Vec<SMatrix<i32, 8, 8>> = Vec::with_capacity(data_vec.len());
            for matrix in data_vec {
                result.push(function(&matrix))
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

    use crate::ppm_parser::read_ppm_from_file;
    #[test]
    fn test_dct_parallel_simple_image() {
        let image = read_ppm_from_file("test/valid_test_8x8.ppm");

        let (y, cb, cr) = crate::parallel_dct::dct(&image, &crate::dct::DCTMode::Arai);

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
    
    #[test]
    fn test_single_channel_simple_image() {
        let image = read_ppm_from_file("test/valid_test_8x8.ppm");

        let y = crate::parallel_dct::dct_single_channel(&image, &crate::dct::DCTMode::Arai);

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

        assert_eq!(y_expected, y);
    }
}
