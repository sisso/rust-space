use std::path::PathBuf;

pub fn get_file(name: &str) -> Result<PathBuf, String> {
    let path_str = format!("{}/../data/{}", env!("CARGO_MANIFEST_DIR"), name);
    let path = PathBuf::from(path_str);
    if !path.exists() {
        Err(format!(
            "file not found at {}/../data/{}",
            env!("CARGO_MANIFEST_DIR"),
            name
        ))
    } else {
        Ok(path)
    }
}
