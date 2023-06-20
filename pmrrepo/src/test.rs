use std::path::Path;
use std::io::Write;
use tempfile::TempDir;
use git2::{
    Oid,
    Repository,
    RepositoryInitOptions,
    Signature,
    Time,
    Tree,
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
    let sig = Signature::now("user", "user@example.com").unwrap();
    let head_id = repo.refname_to_id("HEAD").unwrap();
    let parent = repo.find_commit(head_id).unwrap();
    let commit = repo.commit(Some("HEAD"), &sig, &sig, "commit", &tree, &[&parent]).unwrap();
    (commit, tree_id)
}

fn append_tree_from_objects(
    repo: &Repository,
    tree: Option<&Tree>,
    builder: &mut TreeBuilder,
    gitobj: GitObj,
) {
    match gitobj {
        GitObj::Blob(name, contents) => {
            let oid = repo.blob(
                contents.trim_start_matches('\n').as_bytes()).unwrap();
            builder.insert(name, oid, 0o100644).unwrap();
        },
        GitObj::Commit(name, contents) => {
            // TODO correct this to be an commit actually
            let oid = Oid::from_str(contents).unwrap();
            builder.insert(name, oid, 0o160000).unwrap();
        },
        GitObj::Tree(name, objects) => {
            let subtree = &tree.map_or(None, |t| t.get_path(Path::new(&name))
                .map_or(None, |t| t.to_object(&repo)
                    .map_or(None, |t| t.into_tree().ok())));

            let mut subbuilder = repo.treebuilder(subtree.as_ref()).unwrap();
            for object in objects {
                append_tree_from_objects(
                    &repo, subtree.as_ref(), &mut subbuilder, object);
            }
            let oid = subbuilder.write().unwrap();
            builder.insert(name, oid, 0o040000).unwrap();
        }
    }
}

pub fn append_commit_from_objects(
    repo: &Repository,
    timestamp: Option<i64>,
    message: Option<&str>,
    objects: Vec<GitObj>,
) -> (Oid, Oid) {
    let head_id = repo.refname_to_id("HEAD").unwrap();
    let parent = repo.find_commit(head_id).unwrap();
    let tree = parent.tree().unwrap();
    let mut builder = repo.treebuilder(Some(&tree)).unwrap();
    for objects in objects {
        append_tree_from_objects(&repo, Some(&tree), &mut builder, objects);
    }
    let tree_id = builder.write().unwrap();
    let tree = repo.find_tree(tree_id).unwrap();
    let sig = match timestamp {
        Some(ts) => Signature::new(
            "user", "user@example.com", &Time::new(ts, 0)).unwrap(),
        None => Signature::now(
            "user", "user@example.com").unwrap(),
    };
    let commit = repo.commit(
        Some("HEAD"), &sig, &sig, message.unwrap_or("commit"),
        &tree, &[&parent],
    ).unwrap();
    (commit, tree_id)
}

#[test]
fn smoke_test_append_commit_from_objects() {
    fn assert_blob(repo: &Repository, path: &str, answer: &str) {
        let tree = repo.revparse_single("HEAD").unwrap()
            .as_commit().unwrap()
            .tree().unwrap();
        let file = tree.get_path(Path::new(&path)).unwrap();
        let object = file.to_object(&repo).unwrap();
        let blob = object.as_blob().unwrap();
        assert_eq!(std::str::from_utf8(blob.content()).unwrap(), answer);
    }

    fn assert_commit(repo: &Repository, path: &str, answer: &str) {
        let tree = repo.revparse_single("HEAD").unwrap()
            .as_commit().unwrap()
            .tree().unwrap();
        let target = tree.get_path(Path::new(&path)).unwrap();
        assert_eq!(target.id(), Oid::from_str(answer).unwrap());
    }

    let (td, repo) = crate::test::repo_init(
        None, None, true, Some(1666666666));
    let (commit, tree_id) = crate::test::append_commit_from_objects(
        &repo, Some(1666666700), None,
        vec![
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

    assert_blob(&repo_check, "some_dir/nested/file_a", "file_a in nested");

    // This won't actually resolve to any valid submodule given the
    // above example construct due to the lack of `.gitmodules` file,
    // but given this is a bare repo the test is to be sure that this
    // commit object reference is injected.
    assert_commit(
        &repo_check, "some_gitmodule",
        "0123456789012345678012345678012345678901");

    let (_, _) = crate::test::append_commit_from_objects(
        &repo_check, Some(1666666800), None,
        vec![
            crate::test::GitObj::Blob("new_file", "\na new_file\n"),
            crate::test::GitObj::Tree("some_dir", vec![
                crate::test::GitObj::Blob("file2", "file2 modified"),
                crate::test::GitObj::Blob("file3", "file3 is new"),
                crate::test::GitObj::Tree("nested", vec![
                    crate::test::GitObj::Blob("file_a", "file_a modified"),
                    crate::test::GitObj::Blob("file_c", "file_c is new"),
                ]),
            ]),
        ],
    );

    // first newline trimed out (helps with formatting)
    assert_blob(&repo_check, "new_file", "a new_file\n");
    assert_blob(&repo_check, "some_dir/nested/file_a", "file_a modified");
    assert_blob(&repo_check, "some_dir/nested/file_b", "file_b in nested");
    assert_blob(&repo_check, "some_dir/nested/file_c", "file_c is new");
    assert_commit(
        &repo_check, "some_gitmodule",
        "0123456789012345678012345678012345678901");

}
