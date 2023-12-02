use std::slice::Chunks;
use std::sync::mpsc::{self, Receiver};
use std::thread::{self, JoinHandle};

use crate::downsample::downsample_rows;

/// Down-sample a color channel of an image.
/// `a` and `b` are expected to fit the first two parts of standard subsampling notation: https://en.wikipedia.org/wiki/Chroma_subsampling
///
/// # Arguments
///
/// * `channel`: The color channel to downsample.
/// * `a`: `a` as per the standard subsampling notation.
/// * `b`: `b` as per the standard subsampling notation.
/// * `downsample_vertical`: Whether every set of two rows should also be combined into one (vertical downsampling).
///
/// # Examples
///```
/// let result_cb = downsample_channel(&self.data2, a, b, c != 0);
/// ```
pub fn downsample_channel(
    channel: &Vec<Vec<u16>>,
    a: usize,
    b: usize,
    downsample_vertical: bool,
) -> Vec<Vec<u16>> {
    let len = if downsample_vertical {
        channel.len() / 2
    } else {
        channel.len()
    };
    let (y_handles, y_receivers) = spawn_threads_for_channel(channel, a, b, downsample_vertical);
    join_and_receive_threads_for_channel(y_handles, y_receivers, len)
}

/// Spawn the worker threads for each channel.
/// The channel data is split up into chunks of equal size,
/// each of which is then passed into its own thread.
///
/// # Arguments
/// * `channel`: The color channel to downsample.
/// * `a`: `a` as per the standard subsampling notation.
/// * `b`: `b` as per the standard subsampling notation.
/// * `downsample_vertical`: Whether every set of two rows should also be combined into one (vertical downsampling).
fn spawn_threads_for_channel(
    channel: &Vec<Vec<u16>>,
    a: usize,
    b: usize,
    downsample_vertical: bool,
) -> (Vec<JoinHandle<()>>, Vec<Receiver<Vec<Vec<u16>>>>) {
    let thread_count = thread::available_parallelism().unwrap().get();
    let mut chunk_size = channel.len() / thread_count + 1;
    // ensure that chunk_size is divisible by two - otherwise, vertical downsampling breaks
    if chunk_size % 2 == 1 {
        chunk_size += 1
    };
    let data_vecs: Chunks<'_, Vec<u16>> = channel.chunks(chunk_size);
    let mut handles: Vec<JoinHandle<()>> = Vec::with_capacity(thread_count);
    let mut receivers: Vec<Receiver<Vec<Vec<u16>>>> = Vec::with_capacity(thread_count);

    for data in data_vecs {
        let (tx, rx) = mpsc::channel::<Vec<Vec<u16>>>();
        // slow copy because directly using `data` leads to borrow issues. maybe fixable with lifetimes?
        let data_vec = data.to_owned();

        let handle = thread::spawn(move || {
            let mut result: Vec<Vec<u16>> = Vec::with_capacity(data_vec.len());
            for (index, upper_row) in data_vec.iter().enumerate().step_by(2) {
                let lower_row = if index + 1 < data_vec.len() {
                    &data_vec[index + 1]
                } else {
                    &data_vec[index]
                };

                let (final_row, final_lower_row) =
                    downsample_rows(upper_row, lower_row, a, b, downsample_vertical);

                result.push(final_row);
                if !downsample_vertical && index + 1 < data_vec.len() {
                    result.push(final_lower_row);
                }
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
/// * `capacity`: The amount of vectors in the result. Used to avoid having to reallocate.
fn join_and_receive_threads_for_channel(
    handles: Vec<JoinHandle<()>>,
    receivers: Vec<Receiver<Vec<Vec<u16>>>>,
    capacity: usize,
) -> Vec<Vec<u16>> {
    let mut result: Vec<Vec<u16>> = Vec::with_capacity(capacity);
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
    use crate::parallel_downsample::downsample_channel;

    // #[test]
    // fn test_downsample_parallel_simple_image() {
    //     let image = read_ppm_from_file("test/valid_test_8x8.ppm");
    //     downsample_channel(image.channel1(), 4, 2, false);
    // }

    #[test]
    fn test_downsample_parallel_channel_vertical() {
        let input_channel = vec![
            vec![1, 2, 3, 4],
            vec![5, 6, 7, 8],
            vec![9, 10, 11, 12],
            vec![13, 14, 15, 16],
        ];

        let expected_output: Vec<Vec<u16>> = vec![vec![3, 5], vec![11, 13]];

        let result = downsample_channel(&input_channel, 4, 2, true);

        assert_eq!(expected_output, result);
    }

    #[test]
    fn test_downsample_parallel_channel_horizontal() {
        let input_channel = vec![
            vec![1, 2, 3, 4],
            vec![5, 6, 7, 8],
            vec![9, 10, 11, 12],
            vec![13, 14, 15, 16],
        ];

        let expected_output: Vec<Vec<u16>> =
            vec![vec![1, 3], vec![5, 7], vec![9, 11], vec![13, 15]];

        let result = downsample_channel(&input_channel, 4, 2, false);

        assert_eq!(expected_output, result);
    }

    #[test]
    fn test_downsample_parallel_channel_no_change() {
        let input_channel = vec![
            vec![1, 2, 3, 4],
            vec![5, 6, 7, 8],
            vec![9, 10, 11, 12],
            vec![13, 14, 15, 16],
        ];

        let result = downsample_channel(&input_channel, 4, 4, false);

        assert_eq!(input_channel, result);
    }
}
