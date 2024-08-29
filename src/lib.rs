use std::{
    collections::HashSet,
    fs::{self, remove_dir_all},
    io,
    path::{Path, PathBuf},
};

use anyhow::Result;
use manifest::{Manifest, ManifestEntry};
use sha256::digest;
use store::{get_store_path, ignore_path};
use tracing::{debug, info};

pub mod git;
pub mod manifest;

pub mod store;

/// Synchronizes all protobuf files in the manifest
pub fn sync_protobufs(manifest: &Manifest) -> Result<()> {
    let store_path = get_store_path()?;

    let mut seen_paths = HashSet::new();

    for entry in &manifest.entries {
        info!("Synchronizing protofs from {}", entry.url);

        // Clone the URL into the store
        let key = digest(entry.url.as_bytes());

        let destination = store_path.join(digest(entry.url.as_bytes()));
        if !seen_paths.contains(&key) {
            // Clone down the repo if we haven't already
            seen_paths.insert(key);

            info!("Updating {}", entry.url);
            git::update_repo(&entry.url, &destination, &entry.rev)?;
            info!("Updated {}", entry.url);
        } else {
            info!("Already updated {}", entry.url);
        }

        deploy_protos(destination, entry)?;
    }

    clean_up_old_paths(manifest)?;

    Ok(())
}

fn deploy_protos(source: PathBuf, entry: &ManifestEntry) -> Result<()> {
    let dest_dir = &entry.dest_directory;
    let destination = if let Some(dest_dir) = dest_dir {
        Path::new(dest_dir)
    } else {
        Path::new(&entry.src_directory)
    };
    let source = source.join(&entry.src_directory);
    debug!(
        "Copying proto files from {} to {}",
        source.display(),
        destination.display()
    );

    if destination.exists() {
        debug!("Removing existing files");
        std::fs::remove_dir_all(destination)?;
    }

    std::fs::create_dir_all(destination)?;
    ignore_path(destination)?;

    copy_dir(source, destination)?;

    Ok(())
}

fn clean_up_old_paths(manifest: &Manifest) -> Result<()> {
    let store_path = get_store_path()?;
    let mut seen_paths = HashSet::new();

    for entry in &manifest.entries {
        let key = digest(entry.url.as_bytes());
        seen_paths.insert(key);
    }

    info!("Seen paths: {:?}", seen_paths);

    for entry in std::fs::read_dir(&store_path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            continue;
        }
        let file_name = path.file_name().unwrap().to_str().unwrap();
        if !seen_paths.contains(file_name) {
            debug!("Removing old repository {}", path.display());
            remove_dir_all(path)?;
        }
    }

    Ok(())
}

fn copy_dir(src: impl AsRef<Path>, dest: impl AsRef<Path>) -> io::Result<()> {
    fs::create_dir_all(&dest)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            copy_dir(entry.path(), dest.as_ref().join(entry.file_name()))?;
        } else {
            debug!("Copying file {}", entry.path().display());
            fs::copy(entry.path(), dest.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}
