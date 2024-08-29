use std::{fs, path::Path};

use anyhow::Result;
use auth_git2::GitAuthenticator;
use git2::{
    build::RepoBuilder, Config, FetchOptions, Object, Oid, RemoteCallbacks, Repository, Status,
    StatusOptions, Statuses,
};
use thiserror::Error;
use tracing::{debug, warn};

#[derive(Debug, Error)]
enum Error {
    #[error("Git error: {0}")]
    GitError(#[from] anyhow::Error),
    #[error("Rev not found: {0}")]
    RevNotFound(String),
    #[error("Repo not found")]
    RepoNotFound,
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
    let mut origin = repo.find_remote("origin")?;
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

    // Check if the reference is a commit
    let target = Oid::from_str(rev);
    let commit = target.and_then(|oid| repo.find_commit(oid));

    let current_ref = repo.head()?.peel_to_commit()?.id();

    let target_object = if commit.is_err() {
        debug!("Rev {} is not a commit. Fetching", rev);
        origin.fetch(&[rev], None, None)?;

        let head_commit = repo
            .find_reference("FETCH_HEAD")?
            .peel_to_commit()
            .map_err(|_| Error::RevNotFound(rev.to_string()))?;
        head_commit.into_object()
    } else {
        // This should always succeed
        commit
            .unwrap_or_else(|_| panic!("COuld not find a commit with rev: {}", rev))
            .into_object()
    };

    debug!("Checking out revision {} -> {}", rev, target_object.id());

    if current_ref == target_object.id() {
        let dirty = is_dirty(&repo)?;
        debug!(
            "Already on revision {}. Dirty? {}",
            target_object.id(),
            dirty
        );
        if dirty {
            hard_reset(&repo, &target_object)?;
        }
        return Ok(());
    }

    repo.checkout_tree(&target_object, None)?;
    let dirty = is_dirty(&repo)?;

    if dirty {
        debug!("Repo is ditry. Hard resetting");
        hard_reset(&repo, &target_object)?;
    }

    if let Err(e) = repo.set_head_detached(target_object.id()) {
        debug!("Failed to set head to revision: {}", e);
    }

    debug!("Checked out revision {} {}", rev, target_object.id());

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

fn is_dirty(repo: &Repository) -> Result<bool> {
    Ok(!get_status(repo)?.is_empty())
}

fn hard_reset(repo: &Repository, object: &Object) -> Result<()> {
    debug!("Hard resetting to {}", object.id());

    repo.reset(object, git2::ResetType::Hard, None)?;

    let statuses = get_status(repo)?;

    let repo_root = repo.path().parent();
    match repo_root {
        None => {
            return Err(Error::RepoNotFound.into());
        }
        Some(repo_root) => {
            if !statuses.is_empty() {
                debug!("Repo is dirty after reset. Performing clean");
                statuses
                    .iter()
                    .filter(|s| s.status() == Status::WT_NEW)
                    .for_each(|s| {
                        if let Some(path) = s.path() {
                            let path = repo_root.join(path);
                            if path.is_file() {
                                if let Err(e) = fs::remove_file(&path) {
                                    warn!("Unable to delete file {}: {}", path.display(), e);
                                }
                            } else if let Err(e) = fs::remove_dir_all(&path) {
                                warn!("Unable to delete directory {}: {}", path.display(), e)
                            }
                        }
                    });
            }
        }
    }

    Ok(())
}

fn get_status(repo: &Repository) -> Result<Statuses> {
    let mut opts = StatusOptions::new();
    opts.include_untracked(true);

    repo.statuses(Some(&mut opts)).map_err(|e| e.into())
}
