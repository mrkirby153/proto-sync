use std::{path::Path, process::ExitCode};

use anyhow::Result;
use clap::{Parser, Subcommand};
use color_print::cprintln;
use proto_sync::{manifest::Manifest, sync_protobufs};
use tracing::debug;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Optional path to a manifest file
    #[arg(short, long, value_name = "FILE")]
    manifest_file: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Synchronize protobufs from the manifest file
    Sync,
    /// Adds an entry to the manifest file
    Add {
        /// The git URL to add
        url: String,
        /// The revision to add
        rev: String,
        /// The path in the repository to add
        path: String,
        /// The destination path. If not provided, the path will be used
        dest: Option<String>,
    },
    Remove {
        /// The local path to remove
        path: String,

        #[arg(short, long)]
        /// Whether to remove the files from the filesystem
        cleanup: bool,
    },
    /// Lists all entries in the manifest file
    List,
    /// Cleans up all generated paths
    Clean,
}

fn main() -> Result<ExitCode> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    let manifest_file = cli.manifest_file.unwrap_or("proto-sync.toml".into());
    let manifest_file = Path::new(&manifest_file);

    debug!("Loading manifest from {}", manifest_file.display());
    let mut manifest = Manifest::load(manifest_file)?;

    let command = cli.command;

    match command {
        Commands::Sync => {
            debug!("Synchronizing protobufs");
            sync_protobufs(&manifest, None)?;
            cprintln!("[<green>+</>] Synchronization complete");
            Ok(ExitCode::SUCCESS)
        }
        Commands::Add {
            url,
            rev,
            path,
            dest,
        } => {
            let existing = manifest.entries.iter().find(|entry| entry.url == url);

            if existing.is_some() {
                cprintln!(
                    "[<yellow>!</>] An entry with the URL <yellow>{}</> already exists.",
                    url
                );
                return Ok(ExitCode::FAILURE);
            }

            let entry = proto_sync::manifest::ManifestEntry::new(url, rev, path.clone(), dest);
            debug!("Adding entry to manifest: {:?}", entry);
            manifest.add_entry(entry);
            manifest.save(Path::new(&manifest_file))?;
            cprintln!(
                "[<green>+</>] Added entry with path <yellow>{}</> and revision <green>{}</> to manifest. Run <blue>proto-sync sync</> to synchronize the new entry",
                path,
            );
            Ok(ExitCode::SUCCESS)
        }
        Commands::Remove { path, cleanup } => {
            cprintln!("[<red>-</>] Removing entry with path <yellow>{}</>", path);

            let existing = manifest
                .entries
                .iter()
                .find(|entry| entry.get_dest_directory() == path);

            if let Some(entry) = existing {
                let index = manifest
                    .entries
                    .iter()
                    .position(|e| {
                        e.src_directory == entry.src_directory
                            && e.url == entry.url
                            && e.rev == entry.rev
                    })
                    .unwrap();

                debug!("Removed entry from manifest");

                if cleanup {
                    cprintln!("[<blue>-</>] Cleaning up generated files");
                    let dst = entry.get_dest_directory();
                    let path = Path::new(dst);
                    if path.exists() {
                        std::fs::remove_dir_all(path)?;
                    }
                }

                manifest.entries.remove(index);
                manifest.save(manifest_file)?;
            } else {
                cprintln!(
                    "[<yellow>!</>] No entry with path <yellow>{}</> found",
                    path
                );
            }
            manifest.save(Path::new(&manifest_file))?;
            Ok(ExitCode::SUCCESS)
        }
        Commands::List => {
            for entry in manifest.entries {
                let dest_directory = entry.get_dest_directory();
                cprintln!("<yellow>{}</>", dest_directory);
                cprintln!("  URL: <blue>{}</>", entry.url);
                cprintln!("  Revision: <blue>{}</>", entry.rev);
                cprintln!("  Source directory: <yellow>{}</>", entry.src_directory);
            }
            Ok(ExitCode::SUCCESS)
        }
        Commands::Clean => {
            cprintln!("[<yellow>!</>] Cleaning up generated files");

            for entry in &manifest.entries {
                let dest = entry.get_dest_directory();
                let path = Path::new(dest);
                if path.exists() {
                    cprintln!("[<red>-</>] Removing path <yellow>{}</>", path.display());
                    std::fs::remove_dir_all(path)?;
                } else {
                    cprintln!(
                        "[<yellow>!</>] Path <yellow>{}</> does not exist. Skipping",
                        path.display()
                    );
                }
            }

            Ok(ExitCode::SUCCESS)
        }
    }
}
