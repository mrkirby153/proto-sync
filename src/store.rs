use std::io::Write;
use std::{
    fs::{create_dir_all, read_to_string, File},
    path::{Path, PathBuf},
};

use anyhow::Result;

/// Gets the path to the store directory
pub fn get_store_path() -> Result<PathBuf> {
    let path = Path::new(".proto-sync");

    if !path.exists() {
        create_dir_all(path)?;
    }

    Ok(path.into())
}

/// Marks a path as ignored by adding a .gitignore file
pub fn ignore_path(path: &Path) -> Result<()> {
    let gitignore = path.join(".gitignore");
    if gitignore.exists() {
        let data = read_to_string(&gitignore)?;
        if data != "*\n" {
            let mut file = File::create(&gitignore)?;
            file.write_all(b"*\n")?;
            Ok(())
        } else {
            Ok(())
        }
    } else {
        // Write the gitignore file
        let mut file = File::create(&gitignore)?;
        file.write_all(b"*\n")?;
        Ok(())
    }
}
