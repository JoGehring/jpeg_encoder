use std::slice::ChunksMut;

use nalgebra::SMatrix;
use scoped_threadpool::Pool;

use crate::utils::THREAD_COUNT;

/// Perform the DCT on an image.
/// The DCT is performed for each channel in sequence.
/// DCT on a channel is parallelised with as many threads as the system has logical CPUs.
///
/// # Arguments
/// * `image`: The image to calculate the DCT for.
pub fn quantize(
    values: &mut Vec<SMatrix<f32, 8, 8>>,
    q_table: SMatrix<f32, 8, 8>,
    pool: &mut Pool,
) -> Vec<SMatrix<i32, 8, 8>> {
    let chunk_size = (values.len() / *THREAD_COUNT) + 1;
    let chunks: ChunksMut<SMatrix<f32, 8, 8>> = values.chunks_mut(chunk_size);
    pool.scoped(|s| {
        for chunk in chunks {
            s.execute(move || {
                for matrix in chunk {
                    crate::quantization::quantize(matrix, &q_table);
                }
            });
        }
    });
    values
        .iter()
        .map(|mat| mat.try_cast::<i32>().unwrap())
        .collect()
}

#[cfg(test)]
mod tests {
    use nalgebra::SMatrix;
    use scoped_threadpool::Pool;
    use std::thread::available_parallelism;

    use crate::parallel_quantize::quantize;

    fn get_pool() -> Pool {
        let thread_count = available_parallelism().unwrap().get();
        return Pool::new(thread_count as u32);
    }

    #[test]
    fn test_quantize_simple_values_from_slides() {
        let mut pool = get_pool();

        let x_vec = vec![
            581.0, -144.0, 56.0, 17.0, 15.0, -7.0, 25.0, -9.0, -242.0, 133.0, -48.0, 42.0, -2.0,
            -7.0, 13.0, -4.0, 108.0, -18.0, -40.0, 71.0, -33.0, 12.0, 6.0, -10.0, -56.0, -93.0,
            48.0, 19.0, -8.0, 7.0, 6.0, -2.0, -17.0, 9.0, 7.0, -23.0, -3.0, -10.0, 5.0, 3.0, 4.0,
            9.0, -4.0, -5.0, 2.0, 2.0, -7.0, 3.0, -9.0, 7.0, 8.0, -6.0, 5.0, 12.0, 2.0, -5.0, -9.0,
            -4.0, -2.0, -3.0, 6.0, 1.0, -1.0, -1.0,
        ];
        let mut input: Vec<SMatrix<f32, 8, 8>> = vec![SMatrix::from_row_iterator(x_vec.into_iter())];
        let y_vec = vec![
            12, -3, 1, 0, 0, 0, 0, 0, -5, 3, -1, 1, 0, 0, 0, 0,
            2, 0, -1, 1, -1, 0, 0, 0, -1, -2, 1, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];
        let expected: SMatrix<i32, 8, 8> = SMatrix::from_row_iterator(y_vec.into_iter());
        let q_table = crate::quantization::uniform_q_table(50.0);
        let result = quantize(&mut input, q_table, &mut pool);

        assert_eq!(1, result.len());
        assert_eq!(expected, result[0]);
    }
}
