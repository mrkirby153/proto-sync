use std::path::Path;

use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Default)]
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
    dest_directory: Option<String>,
}

impl Manifest {
    /// Loads a manifest from the given path
    pub fn load(path: &Path) -> Result<Self> {
        let contents = match std::fs::read_to_string(path) {
            Ok(contents) => contents,
            Err(err) => {
                if err.kind() == std::io::ErrorKind::NotFound {
                    return Ok(Self::default());
                } else {
                    return Err(err.into());
                }
            }
        };

        let manifest: Manifest = toml::from_str(&contents)?;
        Ok(manifest)
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        let contents = toml::to_string(self)?;
        std::fs::write(path, contents)?;
        Ok(())
    }

    /// Adds an entry to the manifest file
    pub fn add_entry(&mut self, entry: ManifestEntry) {
        self.entries.push(entry);
    }
}

impl ManifestEntry {
    pub fn new(
        url: String,
        rev: String,
        src_directory: String,
        dest_directory: Option<String>,
    ) -> Self {
        Self {
            url,
            rev,
            src_directory,
            dest_directory,
        }
    }

    pub fn get_dest_directory(&self) -> &str {
        self.dest_directory
            .as_deref()
            .unwrap_or(&self.src_directory)
    }
}
