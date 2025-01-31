use std::fs::File;
use std::io::{self, Read};
use walkdir::WalkDir;
fn read_file_to_bytes(file_path: String) -> io::Result<Vec<u8>> {
    let mut file = File::open(file_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    Ok(buffer)
}

pub fn encode_dir(dirloc: &String) -> String {
    //It will take the folder and convert it to a json representation of a .torrent file
    for entry in WalkDir::new(dirloc) {
        let entry = entry.unwrap(); // unwrap the result to get the directory entry
        if entry.file_type().is_dir() {
            continue;
        }
        let bytes = read_file_to_bytes(entry.path().display().to_string());
        println!("Bytes Size:{}", bytes.unwrap().len());
    }
    return String::from("");
}
