use hocon::HoconLoader;
use serde::Deserialize;
use std::path::Path;

pub fn load_file<'a, K: Deserialize<'a>>(path: &Path) -> Result<K, String> {
    if !path.exists() {
        return Err(format!("file not found at {}", path.display()));
    }
    let mut loader = HoconLoader::new().strict().no_system();
    loader = loader.load_file(path).map_err(|err| format!("{:?}", err))?;
    let raw = loader.hocon().map_err(|err| format!("{:?}", err))?;
    raw.resolve().map_err(|err| format!("{:?}", err))
}

pub fn load_str<'a, K: Deserialize<'a>>(content: &str) -> Result<K, String> {
    let mut loader = HoconLoader::new().strict().no_system();
    loader = loader
        .load_str(content)
        .map_err(|err| format!("{:?}", err))?;
    let raw = loader.hocon().map_err(|err| format!("{:?}", err))?;
    raw.resolve().map_err(|err| format!("{:?}", err))
}

pub fn load_hocon_files<'a, T: AsRef<Path>, K: Deserialize<'a>>(dir: T) -> Result<K, String> {
    let mut loader = HoconLoader::new().strict().no_system();

    for path in std::fs::read_dir(dir)
        .map_err(|e| format!("fail to read configuration from provided dir: {:?}", e))?
    {
        loader = loader
            .load_file(path.unwrap().path())
            .map_err(|err| format!("{:?}", err))?;
    }

    let raw = loader.hocon().map_err(|err| format!("{:?}", err))?;
    raw.resolve().map_err(|err| format!("{:?}", err))
}
