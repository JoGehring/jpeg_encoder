use std::io::Write;
use std::{
    f32::consts::{PI, SQRT_2},
    fs::File,
};

pub fn write_dct_constants_file() {
    let arai_c: [f32; 8] = [
        (0f32 * PI / 16f32).cos(),
        (1f32 * PI / 16f32).cos(),
        (2f32 * PI / 16f32).cos(),
        (3f32 * PI / 16f32).cos(),
        (4f32 * PI / 16f32).cos(),
        (5f32 * PI / 16f32).cos(),
        (6f32 * PI / 16f32).cos(),
        (7f32 * PI / 16f32).cos(),
    ];
    let arai_a: [f32; 6] = [
        0.0f32,
        arai_c[4],
        arai_c[2] - arai_c[6],
        arai_c[4],
        arai_c[6] + arai_c[2],
        arai_c[6],
    ];
    let arai_s: [f32; 8] = [
        1f32 / (2f32 * SQRT_2),
        1f32 / (4f32 * arai_c[1]),
        1f32 / (4f32 * arai_c[2]),
        1f32 / (4f32 * arai_c[3]),
        1f32 / (4f32 * arai_c[4]),
        1f32 / (4f32 * arai_c[5]),
        1f32 / (4f32 * arai_c[6]),
        1f32 / (4f32 * arai_c[7]),
    ];

    let mut file = File::create("src/dct_constants.rs").unwrap();
    writeln!(file, "pub const ARAI_A: [f32; 6] = [").unwrap();
    for (i, a) in arai_a.iter().enumerate() {
        let append = if i == 5 { "" } else { "," };
        write!(file, "{}f32", a).unwrap();
        writeln!(file, "{}", append).unwrap();
    }
    writeln!(file, "];").unwrap();

    writeln!(file, "pub const ARAI_S: [f32; 8] = [").unwrap();
    for (i, s) in arai_s.iter().enumerate() {
        let append = if i == 7 { "" } else { "," };
        write!(file, "{}f32", s).unwrap();
        writeln!(file, "{}", append).unwrap();
    }
    writeln!(file, "];").unwrap();
}
