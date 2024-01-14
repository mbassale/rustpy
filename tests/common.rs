use std::fs;
use std::path::PathBuf;

pub fn load_source(filename: &str) -> String {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push(filename);

    fs::read_to_string(path).expect("Unable to read file")
}
