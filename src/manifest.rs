use std::path::Path;

use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Manifest {
    /// A list of manifest entries declaring what protobufs to download
    #[serde(rename = "entry")]
    pub entries: Vec<ManifestEntry>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ManifestEntry {
    /// The git repository URL
    pub url: String,
    /// The revision to checkout
    pub rev: String,
    /// The path in the directory where the protobufs are located
    pub src_directory: String,
    /// The path in the target project where protobufs should be copied to. If left blank, it is assumed to be the same as `src_directory`
    pub dest_directory: Option<String>,
}

impl Manifest {
    /// Loads a manifest from the given path
    pub fn load(path: &Path) -> Result<Self> {
        let contents = std::fs::read_to_string(path)?;
        let manifest: Manifest = toml::from_str(&contents)?;
        Ok(manifest)
    }
}
