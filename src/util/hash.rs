use std::{
    fs::File,
    path::PathBuf, 
    io::Read,
};
use sha1::{Sha1, Digest};

pub fn get_file_hash(path: PathBuf) -> String {
    let mut file = File::open(path).unwrap();
    let mut hasher = Sha1::new();
    let mut buffer = [0; 1024];
    let mut buf_cnt = 0;
    while let Ok(bytes_read) = file.read(&mut buffer) {
        buf_cnt += 1;
        if bytes_read == 0 || buf_cnt >= 102400 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }
    let hash = format!("{:x}", hasher.finalize());
    hash
}

pub fn get_str_hash(str: &str) -> String {
    let mut hasher = Sha1::new();
    hasher.update(str);
    let hash = format!("{:x}", hasher.finalize());
    hash
}