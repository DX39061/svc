mod hash;
mod compress;

pub use hash::{get_file_hash, get_str_hash};
pub use compress::{compress_data, decompress_data};