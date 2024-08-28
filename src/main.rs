use std::path::Path;

use anyhow::Result;

use proto_sync::manifest::Manifest;
use tracing::info;

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let manifest = Manifest::load(Path::new("proto-sync.toml")).unwrap();

    info!("Loaded manifest: {:?}", manifest);

    Ok(())
}
