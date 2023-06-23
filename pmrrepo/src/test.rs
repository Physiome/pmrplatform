use std::{
    collections::HashMap,
    path::Path,
    str::FromStr,
};
use tempfile::TempDir;
use gix::{
    Repository,
    config::tree::{
        Init,
        Key,
    },
    objs::tree,
};

pub enum GitObj<'a> {
    Blob(&'a str, &'a str),
    Commit(&'a str, &'a str),
    Tree(&'a str, Vec<GitObj<'a>>),
}

pub fn repo_init(
    main_branch: Option<&str>,
    target: Option<&Path>,
    timestamp: Option<i64>,
) -> anyhow::Result<(Option<TempDir>, Repository)> {
    let open_opts = gix::open::Options::isolated();
    let open_opts = match main_branch {
        Some(s) => open_opts.config_overrides([
            Init::DEFAULT_BRANCH.validated_assignment_fmt(&s)?]),
        None => open_opts,
    };

    let tempdir = match target {
        Some(_) => None,
        None => Some(tempfile::tempdir()?)
    };
    let repo = gix::ThreadSafeRepository::init_opts(
        target.unwrap_or_else(|| tempdir.as_ref().unwrap().path()),
        gix::create::Kind::Bare,
        gix::create::Options::default(),
        open_opts,
    )?.to_thread_local();
    init_empty_commit(&repo, timestamp)?;
    Ok((tempdir, repo))
}

pub fn init_empty_commit(
    repo: &Repository,
    timestamp: Option<i64>,
) -> anyhow::Result<()> {
    let tree = gix::objs::Tree::empty();
    let tree_id = repo.write_object(&tree)?.detach();
    let signature = gix::actor::Signature {
        name: "user".into(), email: "user@example.com".into(),
        time: match timestamp {
            None => gix::date::Time::now_utc(),
            Some(t) => gix::date::Time::new(t.try_into()?, 0),
        }
    };
    repo.commit_as(
        &signature,
        &signature,
        "HEAD",
        "initial commit",
        tree_id,
        gix::commit::NO_PARENT_IDS,
    )?;
    Ok(())
}

fn append_tree_from_objects<'a>(
    repo: &'a Repository,
    treeref: Option<gix::objs::TreeRef<'a>>,
    gitobjs: Vec<GitObj>,
) -> anyhow::Result<gix::ObjectId> {
    // would rather be using bstr directly but no idea how to use it
    // with HashMap... converting to something easier to work with and
    // stop wasting time trying to figure that out...
    let mut existing: HashMap<String, _> = treeref.map_or_else(
        || [].into(),
        |x| x.entries.into_iter()
            .map(|e| (e.filename.try_into().unwrap(), e))
            .collect(),
    );
    let mut tree = gix::objs::Tree::empty();
    for gitobj in gitobjs {
        match gitobj {
            GitObj::Blob(name, contents) => {
                let oid = repo.write_blob(
                    contents.trim_start_matches('\n').as_bytes()).unwrap().into();
                tree.entries.push(tree::Entry {
                    mode: tree::EntryMode::Blob,
                    oid: oid,
                    filename: name.into(),
                });
                existing.remove(name);
            },
            GitObj::Commit(name, contents) => {
                tree.entries.push(tree::Entry {
                    mode: tree::EntryMode::Commit,
                    oid: gix::ObjectId::from_str(contents).unwrap(),
                    filename: name.into(),
                });
                existing.remove(name);
            },
            GitObj::Tree(name, objects) => {
                let treeref = match existing.remove(name) {
                    None => None,
                    Some(e) => match repo.try_find_object(e.oid) {
                        Err(_) => None,
                        Ok(obj) => match obj {
                            None => None,
                            Some(obj) => Some(obj.into_tree())
                        }
                    }
                };
                let oid = append_tree_from_objects(
                    repo,
                    // XXX nasty surprise if it wasn't a tree previously
                    treeref.as_ref().map(|t| t.decode().unwrap().into()),
                    objects
                )?;
                tree.entries.push(tree::Entry {
                    mode: tree::EntryMode::Tree,
                    oid: oid,
                    filename: name.into(),
                })
            }
        }
    }
    // TODO need to write out existing things...
    for (_, entry) in existing.iter() {
        tree.entries.push((*entry).clone().into());
    }
    tree.entries.sort();
    Ok(repo.write_object(tree)?.detach())
}

pub fn append_commit_from_objects(
    repo: &Repository,
    timestamp: Option<i64>,
    message: Option<&str>,
    objects: Vec<GitObj>,
) -> anyhow::Result<gix::ObjectId> {
    let prev_commit = repo.head_commit()?;
    let tree = prev_commit.tree()?;
    let treeref = tree.decode()?;
    let signature = gix::actor::Signature {
        name: "user".into(), email: "user@example.com".into(),
        time: match timestamp {
            None => gix::date::Time::now_utc(),
            Some(t) => gix::date::Time::new(t.try_into()?, 0),
        }
    };
    Ok(repo.commit_as(
        &signature,
        &signature,
        "HEAD",
        message.unwrap_or("commit"),
        append_tree_from_objects(&repo, Some(treeref), objects)?,
        [prev_commit.id().detach()],
    )?.detach())
}

pub fn commit(
    repo: &Repository,
    files: Vec<(&str, &str)>,
) -> anyhow::Result<gix::ObjectId> {
    append_commit_from_objects(
        repo,
        None,
        None,
        files.into_iter()
            .map(|(n, c)| crate::test::GitObj::Blob(n, c))
            .collect(),
    )
}

#[test]
fn smoke_test_append_commit_from_objects() {
    fn assert_blob(repo: &git2::Repository, path: &str, answer: &str) {
        let tree = repo.revparse_single("HEAD").unwrap()
            .as_commit().unwrap()
            .tree().unwrap();
        let file = tree.get_path(Path::new(&path)).unwrap();
        let object = file.to_object(&repo).unwrap();
        let blob = object.as_blob().unwrap();
        assert_eq!(std::str::from_utf8(blob.content()).unwrap(), answer);
    }

    fn assert_commit(repo: &git2::Repository, path: &str, answer: &str) {
        let tree = repo.revparse_single("HEAD").unwrap()
            .as_commit().unwrap()
            .tree().unwrap();
        let target = tree.get_path(Path::new(&path)).unwrap();
        assert_eq!(target.id(), git2::Oid::from_str(answer).unwrap());
    }

    let (td, repo) = crate::test::repo_init(
        None, None, Some(1666666666)).unwrap();
    let commit = crate::test::append_commit_from_objects(
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
    ).unwrap();

    let path = &td.unwrap();
    let repo_check = git2::Repository::open_bare(path).unwrap();
    assert_eq!(
        format!("{}", repo_check.revparse_single("HEAD").unwrap().id()),
        "b39494b016b98c591125089e5fa0adefa80076f7",
    );
    assert_eq!(
        format!("{}", commit),
        "b39494b016b98c591125089e5fa0adefa80076f7",
    );
    let tree_id = repo_check.revparse_single("HEAD").unwrap()
        .as_commit().unwrap()
        .tree().unwrap().id();
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

    let _ = crate::test::append_commit_from_objects(
        &repo, Some(1666666800), None,
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
    ).unwrap();

    // first newline trimed out (helps with formatting)
    assert_blob(&repo_check, "new_file", "a new_file\n");
    assert_blob(&repo_check, "some_dir/nested/file_a", "file_a modified");
    assert_blob(&repo_check, "some_dir/nested/file_b", "file_b in nested");
    assert_blob(&repo_check, "some_dir/nested/file_c", "file_c is new");
    assert_commit(
        &repo_check, "some_gitmodule",
        "0123456789012345678012345678012345678901");

}
