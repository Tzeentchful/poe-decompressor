extern crate clap;
use clap::{Arg, App};

use std::path::Path;
use std::io::prelude::*;
use std::fs::File;
use std::u32;

extern crate brotli_decompressor;
extern crate pbr;

use pbr::{ProgressBar, Units};

// Equivalent of the ascii: CMP
const COMPRESSED_FLAG: [u8; 3] = [0x43, 0x4d, 0x50];

fn main() {
    let matches = App::new("poe-decompressor")
    .version("1.0")
    .author("Antonio <ant.haze@gmail.com>")
    .about("PoE Brotli Decompressor")
    .arg(Arg::with_name("INPUT")
        .help("Sets the input file to use")
        .required(true)
        .index(1))
        .arg(Arg::with_name("OUTPUT")
        .help("Sets the output file path")
        .required(true)
        .index(2))
    .get_matches();

    println!("Checking file");

    let in_path = Path::new(matches.value_of("INPUT").unwrap());
    let out_path = Path::new(matches.value_of("OUTPUT").unwrap());

    let mut in_file = File::open(in_path).unwrap();

    // Check the first three bytes of the file for the compression flag
    let mut flag = [0u8; 3];
    in_file.read_exact(&mut flag).unwrap();

    if !flag.iter().zip(COMPRESSED_FLAG.iter()).all(|(a, b)| a == b) {
        println!("Not a PoE compressed file!");
        return
    }

    // The next four bytes is the decompressed file size as a u32 in little endian
    let mut raw_data_len = [0u8; 4];
    in_file.read_exact(&mut raw_data_len).unwrap();
    let data_len = u32::from_le_bytes(raw_data_len);

    println!("Data length: {} bytes", data_len);
    println!("Beginning decompression...");

    let mut pb = ProgressBar::new(data_len as u64);
    pb.set_units(Units::Bytes);

    // The rest of the file is the compressed brotli stream
    let mut out_file = File::create(out_path).unwrap();
    let mut reader = brotli_decompressor::Decompressor::new(in_file, 4096);

    let mut buf = [0u8; 4096];
    let mut count = 0u32 as usize;

    loop {
        match reader.read(&mut buf[..]) {
            Err(e) => {
                if let std::io::ErrorKind::Interrupted = e.kind() {
                    continue;
                }
                panic!(e);
            }
            Ok(size) => {
                if size == 0 {
                    break;
                }
                match out_file.write_all(&buf[..size]) {
                    Err(e) => panic!(e),
                    Ok(_) => {
                        count += size;
                        pb.add(size as u64);
                    },
                }
            }
        }
    }

    pb.finish_print("Finished decompression");

    // Validate that we decompressed the same amount of bytes according to the file header
    if count != data_len as usize {
        println!("WARNING: Decompressed bytes did not meet the specified amount!");
    }
}
