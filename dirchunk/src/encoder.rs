use walkdir::WalkDir;
pub fn encode_dir(dirloc: &String) -> String {
    for entry in WalkDir::new(dirloc) {
        println!("{}", entry.unwrap().path().display());
    }
    return String::from("");
}
