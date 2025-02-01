use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::ffi::CStr;
use std::fs::{self, File};
use std::io::{self, Read};
use std::os::raw::c_char;
use std::ptr;
use walkdir::WalkDir;
fn read_file_to_bytes(file_path: &str) -> io::Result<Vec<u8>> {
    let mut file = File::open(file_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    Ok(buffer)
}
#[no_mangle]
fn encode_dir(dirloc: &str) -> String {
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
#[no_mangle]
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

#[no_mangle]
pub extern "C" fn supplier(chunk_data: *const c_char, chunk_size: usize) -> *mut u8 {
    if chunk_data.is_null() {
        return ptr::null_mut();
    }

    // Convert `chunk_data` (C string) to Rust String
    let chunk_data_str = unsafe { CStr::from_ptr(chunk_data) }.to_str().unwrap_or("");

    // Parse JSON input
    let chunk_data: Value = match serde_json::from_str(chunk_data_str) {
        Ok(v) => v,
        Err(_) => return ptr::null_mut(), // Invalid JSON, return null
    };

    let file_path = chunk_data["filepath"].as_str().unwrap_or("");
    let chunk_number = chunk_data["chunkNumber"].as_u64().unwrap_or(0) as usize;
    let expected_sha = chunk_data["sha"].as_str().unwrap_or("");

    // Read the entire file as bytes
    let file_bytes = match read_file_to_bytes(file_path) {
        Ok(bytes) => bytes,
        Err(_) => return ptr::null_mut(), // File error, return null
    };

    let file_size = file_bytes.len();
    let total_chunks = (file_size + chunk_size - 1) / chunk_size; // Compute total chunks

    if chunk_number >= total_chunks {
        return ptr::null_mut(); // Invalid chunk number
    }

    // Extract the required chunk
    let start = chunk_number * chunk_size;
    let end = std::cmp::min(start + chunk_size, file_size);
    let chunk_data = &file_bytes[start..end];

    // Compute SHA256 hash
    let mut hasher = Sha256::new();
    hasher.update(chunk_data);
    let hash_result = format!("{:x}", hasher.finalize());

    // Verify SHA256
    if hash_result != expected_sha {
        return ptr::null_mut(); // Hash mismatch, return null
    }

    // Allocate memory for output and return pointer
    let chunk_vec = chunk_data.to_vec(); // Copy data
    let chunk_ptr = chunk_vec.as_ptr();
    std::mem::forget(chunk_vec); // Prevent Rust from deallocating

    chunk_ptr as *mut u8 // Return pointer to chunk
}
