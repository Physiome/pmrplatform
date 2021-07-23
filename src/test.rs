use tempfile::TempDir;
use git2::{Repository, RepositoryInitOptions};

pub fn repo_init(main_branch: Option<&str>) -> (TempDir, Repository) {
    let tempdir = TempDir::new().unwrap();
    let mut opts = RepositoryInitOptions::new();
    opts.initial_head(match main_branch {
        Some(s) => { s },
        None => { "main" },
    });
    let repo = Repository::init_opts(tempdir.path(), &opts).unwrap();
    {
        let mut config = repo.config().unwrap();
        config.set_str("user.name", "user").unwrap();
        config.set_str("user.email", "user@example.com").unwrap();
        let mut index = repo.index().unwrap();
        let id = index.write_tree().unwrap();
        let tree = repo.find_tree(id).unwrap();
        let sig = repo.signature().unwrap();
        repo.commit(Some("HEAD"), &sig, &sig, "initial commit", &tree, &[]).unwrap();
    }
    (tempdir, repo)
}
