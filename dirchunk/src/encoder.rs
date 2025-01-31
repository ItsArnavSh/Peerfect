use serde_json::json;
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{self, Read};
use walkdir::WalkDir;

fn read_file_to_bytes(file_path: &str) -> io::Result<Vec<u8>> {
    let mut file = File::open(file_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    Ok(buffer)
}

pub fn encode_dir(dirloc: &String) -> String {
    let chunk_size = 1024 * 1024; // 1MB chunk size
    let mut files = Vec::new();

    for entry in WalkDir::new(dirloc) {
        let entry = entry.unwrap(); // Unwrap the result to get the directory entry
        if entry.file_type().is_dir() {
            continue;
        }

        let file_path = entry.path().display().to_string();
        let bytes = read_file_to_bytes(&file_path).unwrap();
        let file_size = bytes.len();
        let no_of_chunks = ((file_size as f32) / (chunk_size as f32)).ceil() as usize;

        let mut chunks = Vec::new();
        for i in 0..no_of_chunks {
            let start = i * chunk_size;
            let end = std::cmp::min(start + chunk_size, file_size);
            let chunk_data = &bytes[start..end];
            let mut hasher = Sha256::new();
            hasher.update(chunk_data);
            let result = hasher.finalize();
            chunks.push(format!("{:x}", result));
        }

        files.push(json!({
            "path": file_path,
            "chunks": chunks
        }));
    }

    let result_json = json!({
        "version": 1,
        "files": files
    });
    println!("{}", result_json.to_string());
    result_json.to_string()
}
