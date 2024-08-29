use std::{
    collections::HashSet,
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

    for entry in std::fs::read_dir(source)? {
        let entry = entry?;
        let path = entry.path();
        let file_name = path.file_name().unwrap().to_str().unwrap();
        debug!("Copying {}", file_name);
        std::fs::copy(&path, destination.join(file_name))?;
    }

    Ok(())
}
