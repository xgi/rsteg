# rsteg

rsteg is a simple steganography utility for encoding files in images, written in Rust.

## Overview
The payload file is processed as a series of individual bits, which replace the least significant bit of each color channel (RGBA) of each pixel in the cover image. It treats each pixel as containing 1 byte of information for each channel, meaning that 1 bit of payload data are encoded per channel per pixel.

The first 32 bits of encoded data are used to represent an integer which provides the size of the payload, in bytes. This is used to determine the cutoff for extracting the payload.

## Usage
To install dependencies and build the project, run `cargo build [--release]` in the project directory. The `rsteg` binary will be located in the `targets` directory.

```
Usage: ./rsteg <encode/decode> [options]

Options:
    -i input_image      input image filename
    -f payload_file     payload filename
    -o output_file      output filename (image OR payload)
    -c channels         optional number of color channels (def: 4)
    -h, --help          print this help menu
```

Examples:
```bash
./rsteg encode -i test_cover.jpg -f test_payload.txt -o processed.png
./rsteg decode -i processed.png -o retrieved_payload.txt
```
