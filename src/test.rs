use std::path::Path;
use tempfile::TempDir;
use git2::{Oid, Repository, RepositoryInitOptions};

pub fn repo_init(
    main_branch: Option<&str>,
    target: Option<&Path>,
) -> (Option<TempDir>, Repository) {

    let mut opts = RepositoryInitOptions::new();
    opts.initial_head(match main_branch {
        Some(s) => { s },
        None => { "main" },
    });
    let (tempdir, repo) = match target {
        Some(init_path) => (None, Repository::init_opts(init_path, &opts).unwrap()),
        None => {
            let tempdir = TempDir::new().unwrap();
            let init_path = tempdir.path().to_path_buf();
            (Some(tempdir), Repository::init_opts(init_path, &opts).unwrap())
        },
    };
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

pub fn commit(repo: &Repository, filename: &str) -> (Oid, Oid) {
    let mut index = repo.index().unwrap();
    let root = repo.path().parent().unwrap();
    std::fs::File::create(&root.join(filename)).unwrap();
    index.add_path(std::path::Path::new(filename)).unwrap();

    let tree_id = index.write_tree().unwrap();
    let tree = repo.find_tree(tree_id).unwrap();
    let sig = repo.signature().unwrap();
    let head_id = repo.refname_to_id("HEAD").unwrap();
    let parent = repo.find_commit(head_id).unwrap();
    let commit = repo.commit(Some("HEAD"), &sig, &sig, "commit", &tree, &[&parent]).unwrap();
    (commit, tree_id)
}
