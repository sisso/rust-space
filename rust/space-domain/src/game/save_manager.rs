use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// manages list of save games in file, its knows nothing about save game context, but can
/// potentially write some metadata
pub struct SaveManager {
    path: PathBuf,
}

#[derive(Clone, Debug)]
pub struct SaveFile {
    pub filename: String,
    pub modified: SystemTime,
}

impl SaveManager {
    pub fn new(path: &Path) -> Result<Self, &'static str> {
        if !path.exists() {
            log::warn!("path {:?} does not exists", path);
            return Err("path does not exist");
        }

        if !path.is_dir() {
            log::warn!("path {:?} is not a directory", path);
            return Err("path is not a directory");
        }

        Ok(Self { path: path.into() })
    }

    pub fn list_saves(&self) -> Result<Vec<SaveFile>, &'static str> {
        let mut files = vec![];
        for entry in map_err(std::fs::read_dir(&self.path), "fail to read save directory")? {
            let entry = map_err(entry, "fail to read file")?;
            let file_metadata = map_err(entry.metadata(), "fail to read file metadata")?;
            if file_metadata.is_file() {
                let filename = entry.file_name().to_string_lossy().to_string();
                let modified = map_err(file_metadata.modified(), "fail to get file modified")?;
                files.push(SaveFile { filename, modified });
            }
        }
        Ok(files)
    }

    pub fn write(&self, file_name: &str, data: String) -> Result<(), &'static str> {
        let path = self.path.join(file_name);
        log::trace!("writing {:?}", path);
        map_err(
            std::fs::write(path, data.as_bytes()),
            "fail to write save file",
        )
    }

    pub fn read(&self, file_name: &str) -> Result<String, &'static str> {
        let path = self.path.join(file_name);
        log::trace!("reading {:?}", path);
        map_err(std::fs::read_to_string(path), "fail to write save file")
    }

    pub fn get_last(&self) -> Result<Option<SaveFile>, &'static str> {
        let mut list = self.list_saves()?;
        list.sort_by(|a, b| {
            let a = a
                .modified
                .duration_since(UNIX_EPOCH)
                .expect("fail to resolve file modified")
                .as_secs();
            let b = b
                .modified
                .duration_since(UNIX_EPOCH)
                .expect("fail to resolve file modified")
                .as_secs();
            b.cmp(&a)
        });
        Ok(list.get(0).cloned())
    }
}

fn map_err<T>(value: std::io::Result<T>, err_msg: &'static str) -> Result<T, &'static str> {
    match value {
        Ok(v) => Ok(v),
        Err(e) => {
            log::warn!("{} by {:?}", err_msg, e);
            Err(err_msg)
        }
    }
}

#[cfg(test)]
mod test {
    use crate::game::save_manager::SaveManager;
    use crate::test::init_trace_log;

    #[test]
    pub fn test_save_manager() {
        init_trace_log();

        let tmp_dir = commons::fs::create_tmp_dir("rust_space_save_manager").unwrap();
        log::trace!("testing on dir {:?}", tmp_dir);

        let mut saves = SaveManager::new(&tmp_dir).unwrap();
        assert_eq!(0, saves.list_saves().unwrap().len());

        let file1 = "01.txt";
        let file2 = "02.txt";

        saves.write(file1, "0".to_string()).unwrap();
        assert_eq!(1, saves.list_saves().unwrap().len());
        assert_eq!(file1, saves.list_saves().unwrap()[0].filename);
        assert_eq!("0", &saves.read(file1).unwrap());
        assert_eq!(file1, saves.get_last().unwrap().unwrap().filename);

        saves.write(file2, "1".to_string()).unwrap();
        assert_eq!(2, saves.list_saves().unwrap().len());
        assert_eq!("0", &saves.read(file1).unwrap());
        assert_eq!("1", &saves.read(file2).unwrap());
        assert_eq!(file2, saves.get_last().unwrap().unwrap().filename);

        // std::thread::sleep(Duration::from_secs(1));
        //
        // saves.write(file1, "0".to_string()).unwrap();
        // assert_eq!(file1, saves.get_last().unwrap().unwrap().filename);
    }
}
