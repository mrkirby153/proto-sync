use std::{
    collections::HashSet,
    fs::{self, remove_dir_all},
    path::{Path, PathBuf},
};

use anyhow::Result;
use color_print::cprintln;
use manifest::{Manifest, ManifestEntry};
use sha256::digest;
use store::{get_store_path, ignore_path};
use tracing::{debug, info};

pub mod git;
pub mod manifest;

pub mod build;
pub mod store;

pub struct SyncOptions {
    pub base_path: PathBuf,
    pub ignore_generated: bool,
}

/// Synchronizes all protobuf files in the manifest
pub fn sync_protobufs(
    manifest: &Manifest,
    sync_options: Option<SyncOptions>,
) -> Result<Vec<Box<Path>>> {
    let store_path = get_store_path()?;
    let sync_options = sync_options.unwrap_or_default();

    let mut seen_paths = HashSet::new();

    let mut generated_protos = Vec::new();

    for entry in &manifest.entries {
        cprintln!(
            "[<blue>-</>] Synchronizing protobufs from <yellow>{}</>",
            entry.url
        );

        // Clone the URL into the store
        let key = digest(entry.url.as_bytes());

        let destination = store_path.join(digest(entry.url.as_bytes()));
        if !seen_paths.contains(&key) {
            // Clone down the repo if we haven't already
            seen_paths.insert(key);

            cprintln!("[<blue>-</>] Updating repository <yellow>{}</>", entry.url);
            git::update_repo(&entry.url, &destination, &entry.rev)?;
            cprintln!(
                "[<green>-</>] Updated <yellow>{}</> to <blue>{}",
                entry.url,
                entry.rev
            );
        } else {
            cprintln!(
                "[<yellow>!</>] <yellow>{}</> has already been updated. Skipping...",
                entry.url
            );
        }

        let result = deploy_protos(
            destination,
            entry,
            &sync_options.base_path,
            sync_options.ignore_generated,
        )?;
        generated_protos.extend(result);
    }

    clean_up_old_paths(manifest)?;

    Ok(generated_protos)
}

fn deploy_protos(
    source: PathBuf,
    entry: &ManifestEntry,
    base_path: &Path,
    ignore: bool,
) -> Result<Vec<Box<Path>>> {
    let destination = base_path.join(entry.get_dest_directory());
    let source = source.join(&entry.src_directory);

    cprintln!(
        "[<blue>-</>] Copying proto files from <yellow>{}</> to <yellow>{}</>",
        source.display(),
        destination.display()
    );

    if destination.exists() {
        debug!("Removing existing files");
        std::fs::remove_dir_all(&destination)?;
    }

    std::fs::create_dir_all(&destination)?;
    if ignore {
        ignore_path(destination.as_path())?;
    }

    let returned_paths = copy_dir(&source, &destination)?;

    Ok(returned_paths)
}

fn clean_up_old_paths(manifest: &Manifest) -> Result<()> {
    let store_path = get_store_path()?;
    let mut seen_paths = HashSet::new();

    for entry in &manifest.entries {
        let key = digest(entry.url.as_bytes());
        seen_paths.insert(key);
    }

    for entry in std::fs::read_dir(&store_path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            continue;
        }
        let file_name = path.file_name().unwrap().to_str().unwrap();
        if !seen_paths.contains(file_name) {
            cprintln!(
                "[<red>-</>] Removing old repository <yellow>{}</>",
                path.display()
            );
            remove_dir_all(path)?;
        }
    }

    Ok(())
}

fn copy_dir(src: impl AsRef<Path>, dest: impl AsRef<Path>) -> Result<Vec<Box<Path>>> {
    fs::create_dir_all(&dest)?;

    let mut to_return: Vec<Box<Path>> = Vec::new();

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let dest = dest.as_ref().join(entry.file_name());
        if file_type.is_dir() {
            let result = copy_dir(entry.path(), dest)?;
            to_return.extend(result);
        } else {
            debug!("Copying file {}", entry.path().display());
            fs::copy(entry.path(), &dest)?;
            to_return.push(Box::from(dest));
        }
    }
    Ok(to_return)
}

impl Default for SyncOptions {
    fn default() -> Self {
        Self {
            base_path: PathBuf::from("."),
            ignore_generated: true,
        }
    }
}
