use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::{self, Read};
use walkdir::WalkDir;

fn read_file_to_bytes(file_path: &str) -> io::Result<Vec<u8>> {
    let mut file = File::open(file_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    Ok(buffer)
}

pub fn encode_dir(dirloc: &str) -> String {
    let chunk_size = 1024 * 1024; // 1MB chunk size
    let mut files = Vec::new();

    for entry in WalkDir::new(dirloc) {
        let entry = entry.unwrap();
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
        "chunk_size": chunk_size,
        "files": files
    });
    result_json.to_string()
}

pub fn verify(dirloc: &str, torrent_file: &str) {
    let torrent = fs::read_to_string(torrent_file).expect("Failed to read torrent file");
    let torrent_val: Value = serde_json::from_str(&torrent).expect("Invalid JSON format");
    let expected_files = torrent_val["files"].as_array().unwrap();

    let generated_torrent_json = encode_dir(dirloc);
    let generated_torrent: Value = serde_json::from_str(&generated_torrent_json).unwrap();
    let generated_files = generated_torrent["files"].as_array().unwrap();

    for (expected_file, generated_file) in expected_files.iter().zip(generated_files.iter()) {
        let expected_chunks = expected_file["chunks"].as_array().unwrap();
        let generated_chunks = generated_file["chunks"].as_array().unwrap();
        let file_path = expected_file["path"].as_str().unwrap();

        for (i, (expected_chunk, generated_chunk)) in expected_chunks
            .iter()
            .zip(generated_chunks.iter())
            .enumerate()
        {
            if expected_chunk == generated_chunk {
                println!("File {} chunk {} fine", file_path, i);
            } else {
                println!("File {} chunk {} mismatch!", file_path, i);
            }
        }
    }
    println!("Verification complete");
}
