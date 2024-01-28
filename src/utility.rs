use std::fs::File;
use std::io::Read;

use vulkano::shader::spirv::bytes_to_words;

pub fn read_file_to_words(path: &str) -> Vec<u32> {
    let mut file = File::open(path).unwrap();
    let mut buffer = vec![0u8; file.metadata().unwrap().len() as usize];
    file.read(buffer.as_mut_slice()).unwrap();
    bytes_to_words(buffer.as_slice()).unwrap().to_vec()
}
