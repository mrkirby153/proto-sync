use std::path::Path;

use anyhow::Result;

use proto_sync::{
    manifest::Manifest,
    store::{get_store_path, ignore_path},
    sync_protobufs,
};

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let manifest = Manifest::load(Path::new("proto-sync.toml")).unwrap();

    let store_path = get_store_path()?;
    ignore_path(&store_path)?;

    sync_protobufs(&manifest, None)?;

    Ok(())
}
