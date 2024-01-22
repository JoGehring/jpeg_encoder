# JPEG_Encoder

This is a somewhat simple tool converting PPM images to JPEG, created as a course project at the University of Applied Sciences Würzburg-Schweinfurt. It was mainly made and used to teach basics of encoding and compression, as well as practice Rust and various optimisation techniques - this implementation's DCT functions are among the fastest ever created for this course. At time of writing, since the current benchmark (a 4k image) has been in use, the direct DCT is the fastest, the separated DCT is the second fastest and the Arai DCT is tied for fastest DCT implementation created for the course.

A cleaned-up version without unused functions can be found in the "cleanup" branch.

## Usage

If you do want to use this tool, you can do so in 2 ways:

Directly run through Cargo:

```bash
cargo run -r -- /path/to/image.ppm
```

Build and run the binary:

```bash
cargo build -r
./target/release/jpeg_encoder.exe /path/to/image.ppm
```

Note that we recommend building in release mode (``-r``) for performance reasons.

## Project Structure/Encoding Procedure

This summary serves to give a quick understanding of both this project's structure and the general process of JPEG encoding. The modules named in brackets are the ones relevant for the given step of encoding.

0. The image file is parsed ([ppm_parser.rs](./src/ppm_parser.rs)).
1. The image is converted from RGB to YCbCr colours and downsampled ([image.rs](./src/image.rs), [downsample.rs](./src/downsample.rs)).
2. The image data is turned into a set of 8x8 matrices, then the DCT for each of them is calculated ([parallel_dct.rs](./src/parallel_dct.rs), [dct.rs](./src/dct.rs), [arai.rs](./src/arai.rs), [image.rs](./src/image.rs))
3. The resulting matrices are quantized and zig-zag-sampled for further processing ([parallel_quantize.rs](./src/parallel_quantize.rs), [quantization.rs](./src/quantization.rs)).
4. The samples are split into their DC (constant, the top left value of the DCT matrix) and AC (non-constant, the remaining values) components. To compensate for the Y channel not being downsampled by the same factor as the Cb and Cr channels, its coefficients are re-ordered to match the order to write to the JPEG file in the end. ([coefficient_encoder.rs](./src/coefficient_encoder.rs), [huffman.rs](./src/huffman.rs), [package_merge.rs](./src/package_merge.rs))
    - The DC coefficients are difference encoded, then category coded. Said categories are huffman encoded, with the Cb and Cr channel sharing their huffman code.
    - The AC coefficients are runtime length encoded, then category coded. Said categories are then huffman encoded, with the Cb and Cr channel sharing their huffman code.
5. Various JPEG header segments are written to a stream representing the final file ([bit_stream.rs](./src/bit_stream.rs), [jpg_writer.rs](./src/jpg_writer.rs))
6. The image content is written to a stream representing the final file ([bit_stream.rs](./src/bit_stream.rs), [image_data_writer.rs](./src/jpg_writer.rs))
7. The bit stream is flushed into the output file ([bit_stream.rs](./src/bit_stream.rs))
