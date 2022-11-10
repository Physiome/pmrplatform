use std::path::Path;
use std::io::Write;
use tempfile::TempDir;
use git2::{
    Oid,
    Repository,
    RepositoryInitOptions,
    Signature,
    Time,
    TreeBuilder,
};

pub enum GitObj<'a> {
    Blob(&'a str, &'a str),
    Commit(&'a str, &'a str),
    Tree(&'a str, Vec<GitObj<'a>>),
}

pub fn repo_init(
    main_branch: Option<&str>,
    target: Option<&Path>,
    bare: bool,
    timestamp: Option<i64>,
) -> (Option<TempDir>, Repository) {

    let mut opts = RepositoryInitOptions::new();
    opts.initial_head(match main_branch {
            Some(s) => { s },
            None => { "main" },
        })
        .bare(bare);
    let (tempdir, repo) = match target {
        Some(init_path) =>
            (None, Repository::init_opts(init_path, &opts).unwrap()),
        None => {
            let tempdir = TempDir::new().unwrap();
            let init_path = tempdir.path().to_path_buf();
            (Some(tempdir), Repository::init_opts(init_path, &opts).unwrap())
        },
    };
    {
        let mut index = repo.index().unwrap();
        let id = index.write_tree().unwrap();
        let tree = repo.find_tree(id).unwrap();
        let sig = match timestamp {
            Some(ts) => Signature::new(
                "user", "user@example.com", &Time::new(ts, 0)).unwrap(),
            None => Signature::now(
                "user", "user@example.com").unwrap(),
        };
        repo.commit(
            Some("HEAD"), &sig, &sig, "initial commit", &tree, &[]).unwrap();
    }
    (tempdir, repo)
}

pub fn commit(
    repo: &Repository,
    files: Vec<(&str, &str)>,
) -> (Oid, Oid) {
    let mut index = repo.index().unwrap();
    let root = repo.path().parent().unwrap();
    for (filename, contents) in files {
        let mut fd = std::fs::File::create(&root.join(filename)).unwrap();
        fd.write_all(contents.as_bytes()).unwrap();
        index.add_path(std::path::Path::new(filename)).unwrap();
    }

    let tree_id = index.write_tree().unwrap();
    let tree = repo.find_tree(tree_id).unwrap();
    let sig = repo.signature().unwrap();
    let head_id = repo.refname_to_id("HEAD").unwrap();
    let parent = repo.find_commit(head_id).unwrap();
    let commit = repo.commit(Some("HEAD"), &sig, &sig, "commit", &tree, &[&parent]).unwrap();
    (commit, tree_id)
}

fn build_tree_from_objects(
    repo: &Repository,
    tree: &mut TreeBuilder,
    gitobj: GitObj,
) {
    match gitobj {
        GitObj::Blob(name, contents) => {
            let oid = repo.blob(contents.as_bytes()).unwrap();
            tree.insert(name, oid, 0o100644).unwrap();
        },
        GitObj::Commit(name, contents) => {
            // TODO correct this to be an commit actually
            let oid = Oid::from_str(contents).unwrap();
            tree.insert(name, oid, 0o160000).unwrap();
        },
        GitObj::Tree(name, objects) => {
            let mut subtree = repo.treebuilder(None).unwrap();
            for object in objects {
                build_tree_from_objects(&repo, &mut subtree, object);
            }
            let oid = subtree.write().unwrap();
            tree.insert(name, oid, 0o040000).unwrap();
        }
    }
}

pub fn build_commit_from_objects(
    repo: &Repository,
    objects: Vec<GitObj>,
    timestamp: Option<i64>,
) -> (Oid, Oid) {
    let mut root = repo.treebuilder(None).unwrap();
    for objects in objects {
        build_tree_from_objects(&repo, &mut root, objects);
    }
    let tree_id = root.write().unwrap();
    let tree = repo.find_tree(tree_id).unwrap();
    let sig = match timestamp {
        Some(ts) => Signature::new(
            "user", "user@example.com", &Time::new(ts, 0)).unwrap(),
        None => Signature::now(
            "user", "user@example.com").unwrap(),
    };
    let head_id = repo.refname_to_id("HEAD").unwrap();
    let parent = repo.find_commit(head_id).unwrap();
    let commit = repo.commit(
        Some("HEAD"), &sig, &sig, "commit", &tree, &[&parent]).unwrap();
    (commit, tree_id)
}

#[test]
fn test_build_commit_from_objects() {
    let (td, repo) = crate::test::repo_init(
        None, None, true, Some(1666666666));
    let (commit, tree_id) = crate::test::build_commit_from_objects(
        &repo, vec![
            crate::test::GitObj::Blob("some_file", "a blob"),
            crate::test::GitObj::Tree("some_dir", vec![
                crate::test::GitObj::Blob("file1", "file1 in some_dir"),
                crate::test::GitObj::Blob("file2", "file2 in some_dir"),
                crate::test::GitObj::Tree("nested", vec![
                    crate::test::GitObj::Blob("file_a", "file_a in nested"),
                    crate::test::GitObj::Blob("file_b", "file_b in nested"),
                ]),
            ]),
            crate::test::GitObj::Commit(
                "some_gitmodule", "0123456789012345678012345678012345678901"),
        ],
        Some(1666666700),
    );

    let path = &td.unwrap();
    let repo_check = Repository::open_bare(path).unwrap();
    assert_eq!(
        format!("{}", repo_check.revparse_single("HEAD").unwrap().id()),
        "b39494b016b98c591125089e5fa0adefa80076f7",
    );
    assert_eq!(
        format!("{}", commit),
        "b39494b016b98c591125089e5fa0adefa80076f7",
    );
    assert_eq!(
        format!("{}", tree_id),
        "7e0875ba237c0897e5cda37dade7fe58fbc92447",
    );

    let tree = repo_check.revparse_single("HEAD").unwrap()
        .as_commit().unwrap()
        .tree().unwrap();
    let file = tree.get_path(Path::new("some_dir/nested/file_a")).unwrap();
    let object = file.to_object(&repo_check).unwrap();
    let blob = object.as_blob().unwrap();
    assert_eq!(
        std::str::from_utf8(blob.content()).unwrap(),
        "file_a in nested",
    );

    // This won't actually resolve to any valid submodule given the
    // above example construct due to the lack of `.gitmodules` file,
    // but given this is a bare repo the test is to be sure that this
    // commit object reference is injected.
    let target = tree.get_path(Path::new("some_gitmodule")).unwrap();
    assert_eq!(
        format!("{}", target.id()),
        "0123456789012345678012345678012345678901",
    );
}
