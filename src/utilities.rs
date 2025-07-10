use anyhow::Result;
use std::fs::DirEntry;
use std::{fs::read_dir, path::Path};

pub mod tera;

#[allow(dead_code)]
pub fn read_all_files(path: &Path) -> Result<Vec<DirEntry>> {
    let mut files = Vec::new();

    for file in read_dir(path)? {
        let file = file?;
        if Path::new(&file.path()).is_dir() {
            files.append(&mut read_all_files(&file.path())?);
        } else {
            files.push(file);
        }
    }

    files.sort_unstable_by_key(|a| a.path());

    Ok(files)
}

pub mod tests {
    pub mod helpers;
}
