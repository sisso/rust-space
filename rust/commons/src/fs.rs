use rand::{thread_rng, RngCore};
use std::path::{Path, PathBuf};

pub fn create_tmp_dir(prefix: &str) -> Result<PathBuf, String> {
    let random_name = format!("{}_{}", prefix, thread_rng().next_u32());
    let tmp_dir = std::env::temp_dir().join(random_name);
    let path = std::path::PathBuf::from(tmp_dir);
    if !path.exists() {
        std::fs::create_dir(&path).map_err(|err| format!("{:?}", err))?;
    }
    Ok(path)
}
