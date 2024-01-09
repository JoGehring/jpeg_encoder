use crate::{
    bit_stream::BitStream,
    coefficient_encoder::CategoryCode, huffman::HuffmanCode,
};

pub fn write_image_data_to_stream(
    stream: &mut BitStream,
    y_dc_encoded: &[(HuffmanCode, CategoryCode)],
    cb_dc_encoded: &[(HuffmanCode, CategoryCode)],
    cr_dc_encoded: &[(HuffmanCode, CategoryCode)],
    y_ac_encoded: &[Vec<(HuffmanCode, CategoryCode)>],
    cb_ac_encoded: &[Vec<(HuffmanCode, CategoryCode)>],
    cr_ac_encoded: &[Vec<(HuffmanCode, CategoryCode)>],
    image_width: u16,
) {
    let y_entries_per_row = (image_width / 8) as usize;
    let mut y_index = 0;
    for cb_cr_index in 0..cb_dc_encoded.len() {
        // we need to write the four y blocks that make up the same square as the cb_cr_index block
        // so y_index always points to the top left y block in that block
        write_data_at_index(stream, y_dc_encoded, y_ac_encoded, y_index);
        // we then want the one to its left
        write_data_at_index(stream, y_dc_encoded, y_ac_encoded, y_index + 1);
        // and the one right below it (so one row further, hence + y_entries_per_row)
        write_data_at_index(stream, y_dc_encoded, y_ac_encoded, y_index + y_entries_per_row);
        // and the one next to that one
        write_data_at_index(stream, y_dc_encoded, y_ac_encoded, y_index + y_entries_per_row + 1);
        y_index += 2;
        if y_index % y_entries_per_row == 0 {
            // when we reach the end of a row, skip one row because it's already been covered
            y_index += y_entries_per_row
        }

        write_data_at_index(
            stream,
            cb_dc_encoded,
            cb_ac_encoded,
            cb_cr_index,
        );
        write_data_at_index(
            stream,
            cr_dc_encoded,
            cr_ac_encoded,
            cb_cr_index,
        );
    }
}

fn write_data_at_index(
    stream: &mut BitStream,
    dc_encoded: &[(HuffmanCode, CategoryCode)],
    ac_encoded: &[Vec<(HuffmanCode, CategoryCode)>],
    index: usize,
) {
    write_dc(stream, dc_encoded, index);
    write_ac(stream, ac_encoded, index);
}

fn write_dc(
    stream: &mut BitStream,
    dc_encoded: &[(HuffmanCode, CategoryCode)],
    index: usize,
) {
    let dc = &dc_encoded[index];
    stream.append_n_bits(dc.0 .1, dc.0 .0);
    stream.append_n_bits(dc.1 .1, dc.1 .0);
}

fn write_ac(
    stream: &mut BitStream,
    ac_encoded: &[Vec<(HuffmanCode, CategoryCode)>],
    index: usize,
) {
    let ac = &ac_encoded[index];
    for value in ac {
        stream.append_n_bits(value.0 .1, value.0 .0);
        stream.append_n_bits(value.1 .1, value.1 .0);
    }
}
