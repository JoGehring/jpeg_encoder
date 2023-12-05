use std::slice::Chunks;
use std::thread;

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
    downsample_internal(channel, a, b, downsample_vertical, len)
}

fn downsample_internal(channel: &Vec<Vec<u16>>, a: usize, b: usize, downsample_vertical: bool, len: usize) -> Vec<Vec<u16>> {
    let thread_count = thread::available_parallelism().unwrap().get();
    let mut chunk_size = channel.len() / thread_count + 1;
    // ensure that chunk_size is divisible by two - otherwise, vertical downsampling breaks
    if chunk_size % 2 == 1 {
        chunk_size += 1
    };
    let chunks: Chunks<'_, Vec<u16>> = channel.chunks(chunk_size);
    thread::scope(|s| {
        let mut result = Vec::with_capacity(len);
        let mut handles = Vec::with_capacity(chunks.len());
        for chunk in chunks {
            handles.push(s.spawn(move || {
                let mut result: Vec<Vec<u16>> = Vec::with_capacity(chunk.len());
                for (index, upper_row) in chunk.iter().enumerate().step_by(2) {
                    let lower_row = if index + 1 < chunk.len() {
                        &chunk[index + 1]
                    } else {
                        &chunk[index]
                    };

                    let (final_row, final_lower_row) =
                        downsample_rows(upper_row, lower_row, a, b, downsample_vertical);

                    result.push(final_row);
                    if !downsample_vertical && index + 1 < chunk.len() {
                        result.push(final_lower_row);
                    }
                }
                result
            }));
        }
        for handle in handles {
            result.extend(handle.join().unwrap());
        }
        result
    })
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
