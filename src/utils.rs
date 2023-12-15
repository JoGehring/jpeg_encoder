use lazy_static::lazy_static;
use std::ops::{Shl, Shr};

lazy_static! {
    pub static ref THREAD_COUNT: usize = std::thread::available_parallelism().unwrap().get();
}

pub fn get_n_bits_at_offset<T: Shl<u8>>(
    value: T,
    number_of_bits: u8,
    offset: u8,
) -> <<T as Shl<u8>>::Output as Shr<u8>>::Output
where
    T::Output: Shr<u8>,
{
    (value << (offset)) >> (8 - number_of_bits) //same as >> (offset + (8 - offset - number_of_bits))
}
