use std::sync::mpsc::{self, Receiver};
use std::thread::{self, JoinHandle};
use nalgebra::SMatrix;

use crate::image::Image;

pub fn dct(
    image: &Image,
) -> (
    Vec<SMatrix<i32, 8, 8>>,
    Vec<SMatrix<i32, 8, 8>>,
    Vec<SMatrix<i32, 8, 8>>,
) {
    let (y_matrices, cb_matrices, cr_matrices) = image.to_matrices();
    // TODO: maybe split to 6 or 12 threads if possible
    // let available_threads = thread::available_parallelism().unwrap().get();
    let (y_handle, y_rx) = spawn_threads_for_channel(y_matrices);
    let (cb_handle, cb_rx) = spawn_threads_for_channel(cb_matrices);
    let (cr_handle, cr_rx) = spawn_threads_for_channel(cr_matrices);

    y_handle.join().unwrap();
    cb_handle.join().unwrap();
    cr_handle.join().unwrap();

    let y_result = y_rx.recv().unwrap();
    let cb_result = cb_rx.recv().unwrap();
    let cr_result = cr_rx.recv().unwrap();

    (y_result, cb_result, cr_result)
}

fn spawn_threads_for_channel(
    channel: Vec<SMatrix<u16, 8, 8>>,
) -> (JoinHandle<()>, Receiver<Vec<SMatrix<i32, 8, 8>>>) {
    let (tx, rx) = mpsc::channel();

    let handle = thread::spawn(move || {
        let mut result: Vec<SMatrix<i32, 8, 8>> = Vec::with_capacity(channel.len());
        for matrix in channel {
            result.push(crate::dct::arai_dct(&matrix))
        }
        tx.send(result).unwrap()
    });

    (handle, rx)
}
