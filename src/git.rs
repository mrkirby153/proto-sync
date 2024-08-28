use std::path::Path;

use anyhow::Result;
use auth_git2::GitAuthenticator;
use git2::{build::RepoBuilder, Config, FetchOptions, Oid, RemoteCallbacks, Repository};
use thiserror::Error;
use tracing::{debug, warn};

#[derive(Debug, Error)]
enum Error {
    #[error("Git error: {0}")]
    GitError(#[from] anyhow::Error),
    #[error("Rev not found: {0}")]
    RevNotFound(String),
}

/// Updates a repository at the given path to the given revision
pub fn update_repo(repo_url: &str, path: &Path, rev: &str) -> Result<()> {
    debug!("Updating repository at {} to revision {}", repo_url, rev);

    let repo = if path.exists() {
        debug!("Repo exists, opening");
        Repository::open(path)?
    } else {
        debug!("Repo does not exist, cloning...");
        let result = clone_repo(repo_url, path);
        match result {
            Ok(repo) => repo,
            Err(e) => {
                return Err(Error::GitError(e).into());
            }
        }
    };

    // Check that the origin is the same
    let origin = repo.find_remote("origin")?;
    let url = origin
        .url()
        .ok_or_else(|| anyhow::anyhow!("No URL for remote"))?;
    if url != repo_url {
        warn!(
            "Repository URL does not match! Expected: {}, Found: {}. Deleting and retrying",
            repo_url, url
        );
        let _ = std::fs::remove_dir_all(path);
        return update_repo(repo_url, path, rev);
    }

    let current_ref = repo.head()?.peel_to_commit()?.id();

    debug!("Currently on {:?}", current_ref);

    let object = match repo.revparse_single(rev) {
        Ok(object) => object,
        Err(e) => {
            debug!("Revision not found, trying to find it as an oid: {}", e);
            let oid = Oid::from_str(rev);
            match oid {
                Ok(oid) => repo.find_commit(oid)?.into_object(),
                Err(_) => {
                    return Err(Error::RevNotFound(rev.to_string()).into());
                }
            }
        }
    };

    debug!("Checking out revision {} -> {}", rev, object.id());

    if current_ref == object.id() {
        debug!("Already on revision {}", current_ref);
        return Ok(());
    }

    if let Err(e) = repo.set_head_detached(object.id()) {
        debug!("Failed to set head to revision: {}", e);
    }
    repo.checkout_tree(&object, None)?;

    debug!("Checkout out revision {}", rev);

    Ok(())
}

fn clone_repo(repo: &str, path: &Path) -> Result<Repository> {
    debug!("Cloning repository at {} to path {}", repo, path.display());

    let authenticator = GitAuthenticator::default();
    let cfg = Config::open_default()?;
    let mut remote_callbacks = RemoteCallbacks::new();

    remote_callbacks.credentials(authenticator.credentials(&cfg));

    let mut fetch_options = FetchOptions::new();
    fetch_options.remote_callbacks(remote_callbacks);

    let mut builder = RepoBuilder::new();
    builder.fetch_options(fetch_options);

    Ok(builder.clone(repo, path)?)
}
