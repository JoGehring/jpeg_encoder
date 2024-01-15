use lazy_static::lazy_static;

lazy_static! {
    pub static ref THREAD_COUNT: usize = std::thread::available_parallelism().unwrap().get();
}
