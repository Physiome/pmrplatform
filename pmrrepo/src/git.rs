pub use gix::object::Kind;
use std::path::Path;

mod impls;
mod util;

pub use impls::{
    HandleW,
    HandleWR,
    GitResult,
    GitResultTarget,
    WorkspaceGitResult,
    stream_git_result_default,
    stream_git_result_as_json,
    stream_blob,
};

pub(crate) fn fetch_or_clone(
    repo_dir: &Path,
    remote_url: &str,
) -> Result<(), error::FetchClone> {
    // using libgit2 as mature protocol support is desired.
    info!("Syncing local {repo_dir:?} with remote <{remote_url}>...");
    let repo_check = git2::Repository::open_bare(&repo_dir);
    match repo_check {
        Ok(repo) => {
            info!("Found existing repo at {repo_dir:?}, synchronizing...");
            let mut remote = repo.find_remote("origin")
                .map_err(|e| error::FetchClone::Libgit2(e))?;
            match remote.fetch(&[] as &[&str], None, None) {
                Ok(_) => info!("Repository synchronized"),
                Err(e) => return Err(error::FetchClone::Message(e.to_string())),
            };
        }
        Err(ref e) if e.class() == git2::ErrorClass::Repository => {
            return Err(error::FetchClone::Message(
                "expected repo_dir be a bare repo".to_string()
            ));
        }
        Err(_) => {
            info!("Cloning new repository at {repo_dir:?}...");
            let mut builder = git2::build::RepoBuilder::new();
            builder.bare(true);
            match builder.clone(remote_url, &repo_dir) {
                Ok(_) => info!("Repository cloned"),
                Err(e) => return Err(error::FetchClone::Message(
                    format!("fail to clone: {e}")
                )),
            };
        }
    }
    Ok(())
}

pub(crate) mod error {
    pub(crate) enum FetchClone {
        Libgit2(git2::Error),
        Message(String),
    }
}
