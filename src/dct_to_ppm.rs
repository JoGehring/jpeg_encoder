use std::fs::File;
use std::io::{Error, Write};
use std::usize;

use lazy_static::lazy_static;
use nalgebra::SMatrix;
use regex::Regex;

lazy_static! {
    /// This regex checks correct paths given to the to_ppm function
    /// - We start at the beginning of the path string
    /// - The first group checks for (back-)slashes, in case of unixoid systems zero or forward,
    /// in case of Windows zero to two backslashes
    /// - Then we check for dots, there are zero to two allowed
    /// - then we allow every char which is not forbidden by the OS as part of a file/directory
    /// - This whole block has to appear at least once
    /// - Then we check for the right file suffix and the ending of the string
    static ref PPM_FILEPATH_REGEX: Regex =
        Regex::new(r#"^(([/]{0,1} | [\\]{0,2}){0,1}[.]{0,2}[^,;<>:"|\?\*]+)+(.ppm)$"#).unwrap();
}

pub fn to_ppm(
    data: (
        &Vec<SMatrix<f32, 8, 8>>,
        &Vec<SMatrix<f32, 8, 8>>,
        &Vec<SMatrix<f32, 8, 8>>,
    ),
    height: u16,
    width: u16,
    path: &str,
) -> Result<(), Error> {
    if !PPM_FILEPATH_REGEX.is_match(path) {
        return Err(Error::new(
            std::io::ErrorKind::InvalidInput,
            "File path doesn't match our regex!",
        ));
    }
    if !(height % 8 == 0 && width % 8 == 0) {
        return Err(Error::new(
            std::io::ErrorKind::InvalidInput,
            "Width or height not divisible by 8",
        ));
    }
    let r_values: Vec<_> = data.0.chunks(width as usize / 8usize).collect();
    let g_values: Vec<_> = data.1.chunks(width as usize / 8usize).collect();
    let b_values: Vec<_> = data.2.chunks(width as usize / 8usize).collect();

    let mut file = File::create(path)?;
    writeln!(file, "P3")?;
    write!(file, "{}", width.to_string())?;
    write!(file, " ")?;
    writeln!(file, "{}", height.to_string())?;
    writeln!(file, "{}", 255)?;
    for y in 0..r_values.len() {
        for i in 0..8 {
            for j in 0..width as usize {
                let x = j / 8;
                let x_index = j % 8;
                let r_val = (r_values[y][x][(i, x_index)]) as u16;
                let g_val = (g_values[y][x][(i, x_index)]) as u16;
                let b_val = (b_values[y][x][(i, x_index)]) as u16;
                write!(file, "{} {} {} ", r_val, g_val, b_val)?;
            }
            writeln!(file)?;
        }
    }
    Ok(())
}
