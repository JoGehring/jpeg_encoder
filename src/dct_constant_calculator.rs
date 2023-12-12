use lazy_static::lazy_static;
use nalgebra::SMatrix;
use std::io::Write;
use std::{
    f32::consts::{PI, SQRT_2},
    fs::File,
};

const SQRT_2_DIV_2: f32 = SQRT_2 / 2f32;
const MATRIX_C0: f32 = 1.0 / SQRT_2;

lazy_static! {
    static ref ARAI_C: [f32; 8] = [
        (0f32 * PI / 16f32).cos(),
        (1f32 * PI / 16f32).cos(),
        (2f32 * PI / 16f32).cos(),
        (3f32 * PI / 16f32).cos(),
        (4f32 * PI / 16f32).cos(),
        (5f32 * PI / 16f32).cos(),
        (6f32 * PI / 16f32).cos(),
        (7f32 * PI / 16f32).cos(),
    ];
    static ref ARAI_A: [f32; 6] = [
        0.0f32,
        ARAI_C[4],
        ARAI_C[2] - ARAI_C[6],
        ARAI_C[4],
        ARAI_C[6] + ARAI_C[2],
        ARAI_C[6],
    ];
    static ref ARAI_S: [f32; 8] = [
        1f32 / (2f32 * SQRT_2),
        1f32 / (4f32 * ARAI_C[1]),
        1f32 / (4f32 * ARAI_C[2]),
        1f32 / (4f32 * ARAI_C[3]),
        1f32 / (4f32 * ARAI_C[4]),
        1f32 / (4f32 * ARAI_C[5]),
        1f32 / (4f32 * ARAI_C[6]),
        1f32 / (4f32 * ARAI_C[7]),
    ];
}

pub fn write_dct_constants_file() {
    let mut file = File::create("src/dct_constants.rs").unwrap();
    writeln!(file, "use nalgebra::{{ArrayStorage, SMatrix}};").unwrap();

    write_arai_a(&mut file);

    write_arai_s(&mut file);

    let matrix_a_matrix = matrix_dct_a_matrix();
    let matrix_a_matrix_trans = matrix_a_matrix.transpose();

    write_float_matrix(&mut file, "MATRIX_A_MATRIX", &matrix_a_matrix);
    write_float_matrix(&mut file, "MATRIX_A_MATRIX_TRANS", &matrix_a_matrix_trans);

    write_direct_lut(&mut file);
}

fn write_arai_a(file: &mut File) {
    writeln!(file, "pub const ARAI_A: [f32; 6] = [").unwrap();
    for (i, a) in ARAI_A.iter().enumerate() {
        let append = if i == 5 { "" } else { "," };
        write!(file, "{}f32", a).unwrap();
        writeln!(file, "{}", append).unwrap();
    }
    writeln!(file, "];").unwrap();
}

fn write_arai_s(file: &mut File) {
    writeln!(file, "pub const ARAI_S: [f32; 8] = [").unwrap();
    for (i, s) in ARAI_S.iter().enumerate() {
        let append = if i == 7 { "" } else { "," };
        write!(file, "{}f32", s).unwrap();
        writeln!(file, "{}", append).unwrap();
    }
    writeln!(file, "];").unwrap();
}

fn write_float_matrix(file: &mut File, name: &str, matrix: &SMatrix<f32, 8, 8>) {
    writeln!(file, "pub const {}: SMatrix<f32, 8, 8> = SMatrix::<f32, 8, 8>::from_array_storage(ArrayStorage([", name).unwrap();
    for (idx, column) in matrix.column_iter().enumerate() {
        let append = if idx == 7 { "" } else { "," };
        writeln!(
            file,
            "[{}f32, {}f32, {}f32, {}f32, {}f32, {}f32, {}f32, {}f32]{}",
            column[0],
            column[1],
            column[2],
            column[3],
            column[4],
            column[5],
            column[6],
            column[7],
            append
        )
        .unwrap();
    }
    writeln!(file, "]));").unwrap();
}

fn write_direct_lut(file: &mut File) {
    let lut = direct_dct_lookup_table();
    writeln!(
        file,
        "pub const DIRECT_LOOKUP_TABLE: [[[[f32; 8]; 8]; 8]; 8] = ["
    )
    .unwrap();
    for row in lut {
        writeln!(file, "[").unwrap();
        for row2 in row {
            writeln!(file, "[").unwrap();
            for row3 in row2 {
                writeln!(file, "[").unwrap();
                for (idx, val) in row3.iter().enumerate() {
                    let append = if idx == 7 { "" } else { "," };
                    writeln!(file, "{}f32{}", val, append).unwrap();
                }
                writeln!(file, "],").unwrap();
            }
            writeln!(file, "],").unwrap();
        }
        writeln!(file, "],").unwrap();
    }
    writeln!(file, "];").unwrap();
}

/// The matrix used as A in the matrix approach.
fn matrix_dct_a_matrix() -> SMatrix<f32, 8, 8> {
    let matrix_sqrt_const: f32 = 0.25f32.sqrt();

    let mut a_matrix: SMatrix<f32, 8, 8> = SMatrix::from_element(0.0);
    for k in 0..8 {
        for n in 0..8 {
            let cos_val = (((2 * n + 1) * k) as f32 * PI / 16.0f32).cos();
            a_matrix[(k, n)] = cos_val * matrix_sqrt_const * if k == 0 { MATRIX_C0 } else { 1.0 };
        }
    }
    a_matrix
}

/// LUT for the DCT
fn direct_dct_lookup_table() -> [[[[f32; 8]; 8]; 8]; 8] {
    let mut result = [[[[0f32; 8]; 8]; 8]; 8];
    for i in 0..8 {
        for j in 0..8 {
            for x in 0..8 {
                for y in 0..8 {
                    // multiplications with 2/N, C(i) and C(j) are moved in here for optimisation
                    result[i][j][x][y] = ((((2 * x + 1) * i) as f32 * PI) / 16.0).cos()
                        * ((((2 * y + 1) * j) as f32 * PI) / 16.0).cos()
                        * 0.25; // 2/N
                                // this is semantically the same as new_y /= SQRT_2 - optimised because multiplication is faster than division
                                // new_y/SQRT_2 == new_y*SQRT_2/2, SQRT_2_DIV_2 == SQRT_2/2
                    if i == 0 {
                        // C(i)
                        result[i][j][x][y] *= SQRT_2_DIV_2
                    }
                    if j == 0 {
                        // C(j)
                        result[i][j][x][y] *= SQRT_2_DIV_2
                    }
                }
            }
        }
    }

    result
}
