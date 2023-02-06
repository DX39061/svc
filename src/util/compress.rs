use std::io::{Write, Read};

use flate2::{
    read::ZlibDecoder,
    write::ZlibEncoder,
    Compression
};

pub fn compress_data(data: &[u8]) -> Vec<u8>{
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::fast());
    encoder.write(data).unwrap();
    encoder.finish().unwrap()
}

pub fn decompress_data(data: &[u8]) -> Vec<u8> {
    let mut decoder = ZlibDecoder::new(data);
    let mut decompressd_data = Vec::new();
    decoder.read_to_end(&mut decompressd_data).unwrap();
    decompressd_data
}