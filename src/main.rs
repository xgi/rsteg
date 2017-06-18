extern crate getopts;
extern crate image;
extern crate bit_vec;
extern crate byteorder;

use std::env;
use std::io::prelude::*;
use std::fs::File;
use std::path::Path;
use getopts::Options;
use bit_vec::BitVec;
use image::{
	GenericImage,
	DynamicImage,
	ImageBuffer,
	Rgba
};
use byteorder::{
    BigEndian,
    WriteBytesExt,
    ReadBytesExt
};

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} <encode/decode> [options]", program);
    print!("{}", opts.usage(&brief));
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optopt("i", "", "input image filename", "input_image");
    opts.optopt("f", "", "payload filename", "payload_file");
    opts.optopt("o", "", "output filename (image OR payload)", "output_file");
    opts.optopt("c", "", "optional number of color channels (def: 4)", "channels");
    opts.optflag("h", "help", "print this help menu");
    
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m }
        Err(f) => { panic!(f.to_string()) }
    };
    if matches.opt_present("h") {
        print_usage(&program, opts);
        return;
    }
    
    let opt_input_filename = matches.opt_str("i");
    let opt_payload_filename = matches.opt_str("f");
    let opt_output_filename = matches.opt_str("o");
    let opt_channels = matches.opt_str("c");

    let action = if !matches.free.is_empty() {
        matches.free[0].clone()
    } else {
        print_usage(&program, opts);
        return;
    };
    
    let mut channels: usize = 4;
    if opt_channels.is_some() {
        channels = opt_channels.unwrap().parse().unwrap();
    }

    if action == "encode" {
        if opt_input_filename.is_none() ||
           opt_payload_filename.is_none() ||
           opt_output_filename.is_none() {
            print_usage(&program, opts);
            return;
        } else {
            encode(&opt_input_filename.unwrap(), &opt_payload_filename.unwrap(),
                   &opt_output_filename.unwrap(), channels);
        }
    } else if action == "decode" {
        if opt_input_filename.is_none() ||
           opt_output_filename.is_none() {
            print_usage(&program, opts);
            return;
        } else {
            decode(&opt_input_filename.unwrap(), &opt_output_filename.unwrap(),
                   channels);
        }
    } else {
        print_usage(&program, opts);
        return;
    }
}

pub fn read_file_bytes(filename: &String) -> Vec<u8> {
     let mut file = File::open(&Path::new(filename)).expect("Could not open file");
     let mut buf = Vec::new();
     file.read_to_end(&mut buf);
     return buf;
}

pub fn modify_last_bit(byte: u8, bit: bool) -> u8 {
    // get byte as bitvec
    let mut bv = BitVec::from_bytes(&[byte]);
    // modify the last bit
    bv.set(7, bit);
    // convert back to byte
    return bv.to_bytes()[0];
}

pub fn get_last_bit(byte: u8) -> bool {
    // get byte as bitvec
    let mut bv = BitVec::from_bytes(&[byte]);
    return bv.get(7).unwrap();
}

pub fn combine_bit_vecs(a: &BitVec, b: &BitVec) -> BitVec {
    let mut combined = BitVec::new();
    
    for bit in a.iter() {
        combined.push(bit);
    }
    for bit in b.iter() {
        combined.push(bit);
    }
    
    return combined;
}

pub fn bit_vec_from_u32(x: u32) -> BitVec {
    let mut bytes = vec![];
    bytes.write_u32::<BigEndian>(x).unwrap();
    return BitVec::from_bytes(&bytes);
}

pub fn u32_from_bit_vec(bits: &BitVec) -> u32 {
    let bytes = bits.to_bytes();
    let mut buf = std::io::Cursor::new(bits.to_bytes());
    return buf.read_u32::<BigEndian>().unwrap();
}

pub fn image_from_file(filename: &String) -> DynamicImage {
    return image::open(&Path::new(&filename)).unwrap();
}


pub fn encode(cover_filename: &String, payload_filename: &String,
              output_filename: &String, channels: usize) {
    let cover_img = image_from_file(cover_filename);
    let payload = read_file_bytes(payload_filename);

    let (width, height) = cover_img.dimensions();

    // get file size as bitvec
    let payload_size: u32 = payload.len() as u32;
    let size_bits = bit_vec_from_u32(payload_size);

    // get payload as bitvec
    let payload_bits = BitVec::from_bytes(&payload);

    // combine bitvecs
    let data_bits = combine_bit_vecs(&size_bits, &payload_bits);

    // encode bits into new image
    let mut new_img = ImageBuffer::new(width, height);
    let mut index = 0;
	for (x, y, pixel) in cover_img.pixels() {
		let mut new_pixel = pixel;

        // loop over color channels
        for n in 0..channels {
            if index < data_bits.len() as u32 {
                new_pixel.data[n] = modify_last_bit(new_pixel.data[n],
                                                    data_bits.get(index as usize).unwrap());
            }
            index += 1
        }
		new_img.put_pixel(x, y, new_pixel);
	}
    new_img.save(output_filename).unwrap();
}

pub fn decode(image_filename: &String, output_filename: &String, channels: usize) {
    let img = image::open(&Path::new(image_filename)).unwrap().to_rgba();
    let mut bits = BitVec::new();

	for (x, y, pixel) in img.enumerate_pixels() {
        // loop over color channels
        for n in 0..channels {
            bits.push(get_last_bit(pixel.data[n]));
        }
    }

    // get first 32 bits and remove from payload
    let mut payload_size_bits = BitVec::new();
    for n in 0..32 {
        payload_size_bits.push(bits[n]);
    }
    let payload_size = u32_from_bit_vec(&payload_size_bits);

    // get actual payload bits
    let mut payload_bits = BitVec::new();
    for n in 32..(payload_size*8 + 32) {
        payload_bits.push(bits[n as usize]);
    }
   
    // get payload as string and write to file
    let payload = payload_bits.to_bytes();
    let payload_str = String::from_utf8_lossy(&payload).into_owned();
    let mut file = File::create(output_filename).expect("Unable to create output file");
    file.write(payload_str.as_bytes()).expect("Unable to write to output file");
}
