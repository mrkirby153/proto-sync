use std::path::Path;

use anyhow::Result;

use crate::{
    manifest::Manifest,
    store::{get_store_path, ignore_path},
    sync_protobufs, SyncOptions,
};

/// Synchronizes all protobuf files in the manifest. This function should generally be called from `buidl.rs`
pub fn synchronize_protobufs(manifest_path: &str) -> Result<Vec<Box<Path>>> {
    println!("cargo::rerun-if-changed={}", manifest_path);
    let manifest = Manifest::load(Path::new("proto-sync.toml"))?;

    let store_path = get_store_path()?;
    ignore_path(&store_path)?;

    let out_dir = std::env::var("OUT_DIR").unwrap();

    sync_protobufs(
        &manifest,
        Some(SyncOptions {
            base_path: Path::new(&out_dir).to_path_buf(),
            ignore_generated: false,
        }),
    )
}
