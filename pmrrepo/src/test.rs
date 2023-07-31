use async_trait::async_trait;
use gix::{
    Repository,
    config::tree::{
        Init,
        Key,
    },
    objs::tree,
};
use mockall::mock;
use pmrmodel_base::{
    error::BackendError,
    exposure::{
        Exposure,
        Exposures,
        ExposureFile,
        ExposureFiles,
        ExposureFileView,
        ExposureFileViews,
        traits::{
            ExposureBackend,
            ExposureFileBackend,
            ExposureFileViewBackend,
        },
    },
    workspace::{
        Workspace,
        Workspaces,
        WorkspaceAlias,
        WorkspaceSync,
        WorkspaceSyncStatus,
        WorkspaceTag,
        traits::{
            WorkspaceAliasBackend,
            WorkspaceBackend,
            WorkspaceSyncBackend,
            WorkspaceTagBackend,
        },
    },
};
use std::{
    collections::HashMap,
    path::Path,
    str::FromStr,
};
use tempfile::TempDir;
use textwrap_macros::dedent;

mock! {
    pub Platform {
        async fn exposure_insert(
            &self,
            workspace_id: i64,
            workspace_tag_id: Option<i64>,
            commit_id: &str,
            default_file_id: Option<i64>,
        ) -> Result<i64, BackendError>;
        async fn exposure_list_for_workspace(
            &self,
            workspace_id: i64,
        ) -> Result<Exposures, BackendError>;
        async fn exposure_get_id(
            &self,
            id: i64,
        ) -> Result<Exposure, BackendError>;
        async fn exposure_set_default_file(
            &self,
            id: i64,
            file_id: i64,
        ) -> Result<bool, BackendError>;

        async fn exposure_file_insert(
            &self,
            exposure_id: i64,
            workspace_file_path: &str,
            default_view_id: Option<i64>,
        ) -> Result<i64, BackendError>;
        async fn exposure_file_list_for_exposure(
            &self,
            exposure_id: i64,
        ) -> Result<ExposureFiles, BackendError>;
        async fn exposure_file_get_id(
            &self,
            id: i64,
        ) -> Result<ExposureFile, BackendError>;
        async fn exposure_file_set_default_view(
            &self,
            id: i64,
            file_id: i64,
        ) -> Result<bool, BackendError>;

        async fn exposure_file_view_insert(
            &self,
            exposure_file_id: i64,
            view_task_template_id: i64,
        ) -> Result<i64, BackendError>;
        async fn exposure_file_view_list_for_exposure_file(
            &self,
            exposure_file_id: i64,
        ) -> Result<ExposureFileViews, BackendError>;
        async fn exposure_file_view_get_id(
            &self,
            id: i64,
        ) -> Result<ExposureFileView, BackendError>;
        async fn exposure_file_view_update_view_key(
            &self,
            id: i64,
            view_key: &str,
        ) -> Result<bool, BackendError>;
    }

    #[async_trait]
    impl WorkspaceTagBackend for Platform {
        async fn index_workspace_tag(&self, workspace_id: i64, name: &str, commit_id: &str) -> Result<i64, BackendError>;
        async fn get_workspace_tags(&self, workspace_id: i64) -> Result<Vec<WorkspaceTag>, BackendError>;
    }

    #[async_trait]
    impl WorkspaceBackend for Platform {
        async fn add_workspace(
            &self, url: &str, description: &str, long_description: &str
        ) -> Result<i64, BackendError>;
        async fn update_workspace(
            &self, id: i64, description: &str, long_description: &str
        ) -> Result<bool, BackendError>;
        async fn list_workspaces(&self) -> Result<Workspaces, BackendError>;
        async fn get_workspace_by_id(&self, id: i64) -> Result<Workspace, BackendError>;
        async fn list_workspace_by_url(&self, url: &str) -> Result<Workspaces, BackendError>;
    }

    #[async_trait]
    impl WorkspaceSyncBackend for Platform {
        async fn begin_sync(&self, workspace_id: i64) -> Result<i64, BackendError>;
        async fn complete_sync(&self, id: i64, status: WorkspaceSyncStatus) -> Result<bool, BackendError>;
        async fn get_workspaces_sync_records(&self, workspace_id: i64) -> Result<Vec<WorkspaceSync>, BackendError>;
    }

    #[async_trait]
    impl WorkspaceAliasBackend for Platform {
        async fn add_alias(
            &self,
            workspace_id: i64,
            alias: &str,
        ) -> Result<i64, BackendError>;
        async fn get_aliases(
            &self,
            workspace_id: i64,
        ) -> Result<Vec<WorkspaceAlias>, BackendError>;
    }

}

#[async_trait]
impl ExposureBackend for MockPlatform {
    async fn insert(
        &self,
        workspace_id: i64,
        workspace_tag_id: Option<i64>,
        commit_id: &str,
        default_file_id: Option<i64>,
    ) -> Result<i64, BackendError> {
        self.exposure_insert(workspace_id, workspace_tag_id, commit_id, default_file_id).await
    }
    async fn list_for_workspace(
        &self,
        workspace_id: i64,
    ) -> Result<Exposures, BackendError> {
        self.exposure_list_for_workspace(workspace_id).await
    }
    async fn get_id(
        &self,
        id: i64,
    ) -> Result<Exposure, BackendError> {
        self.exposure_get_id(id).await
    }
    async fn set_default_file(
        &self,
        id: i64,
        file_id: i64,
    ) -> Result<bool, BackendError> {
        self.set_default_file(id, file_id).await
    }
}

#[async_trait]
impl ExposureFileBackend for MockPlatform {
    async fn insert(
        &self,
        exposure_id: i64,
        workspace_file_path: &str,
        default_view_id: Option<i64>,
    ) -> Result<i64, BackendError> {
        self.exposure_file_insert(exposure_id, workspace_file_path, default_view_id).await
    }
    async fn list_for_exposure(
        &self,
        exposure_id: i64,
    ) -> Result<ExposureFiles, BackendError> {
        self.exposure_file_list_for_exposure(exposure_id).await
    }
    async fn get_id(
        &self,
        id: i64,
    ) -> Result<ExposureFile, BackendError> {
        self.exposure_file_get_id(id).await
    }
    async fn set_default_view(
        &self,
        id: i64,
        file_id: i64,
    ) -> Result<bool, BackendError> {
        self.exposure_file_set_default_view(id, file_id).await
    }
}

#[async_trait]
impl ExposureFileViewBackend for MockPlatform {
    async fn insert(
        &self,
        exposure_file_id: i64,
        view_task_template_id: i64,
    ) -> Result<i64, BackendError> {
        self.exposure_file_view_insert(exposure_file_id, view_task_template_id).await
    }
    async fn list_for_exposure_file(
        &self,
        exposure_file_id: i64,
    ) -> Result<ExposureFileViews, BackendError> {
        self.exposure_file_view_list_for_exposure_file(exposure_file_id).await
    }
    async fn get_id(
        &self,
        id: i64,
    ) -> Result<ExposureFileView, BackendError> {
        self.exposure_file_view_get_id(id).await
    }
    async fn update_view_key(
        &self,
        id: i64,
        view_key: &str,
    ) -> Result<bool, BackendError> {
        self.exposure_file_view_update_view_key(id, view_key).await
    }
}


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

pub fn create_repodata() -> (
    TempDir,
    (gix::Repository, Vec<gix::ObjectId>),
    (gix::Repository, Vec<gix::ObjectId>),
    (gix::Repository, Vec<gix::ObjectId>),
) {
    use crate::test::GitObj::{
        Blob,
        Commit,
        Tree,
    };

    let tempdir = TempDir::new().unwrap();
    // import1
    let (_, import1) = crate::test::repo_init(
        None, Some(&tempdir.path().join("1")), Some(1111010101)).unwrap();
    let mut import1_oids = <Vec<gix::ObjectId>>::new();
    let mut import2_oids = <Vec<gix::ObjectId>>::new();
    let mut repodata_oids = <Vec<gix::ObjectId>>::new();

    import1_oids.push(crate::test::append_commit_from_objects(
        &import1, Some(1111010110), Some("readme for import1"), vec![
        Blob("README", dedent!("
        this is import1
        ")),
    ]).unwrap());
    import1_oids.push(crate::test::append_commit_from_objects(
        &import1, Some(1111010111), Some("adding import1"), vec![
        Blob("if1", dedent!("
        if1
        ")),
        Blob("README", dedent!("
        The readme for import1.
        ")),
    ]).unwrap());

    // import2
    let (_, import2) = crate::test::repo_init(
        None, Some(&tempdir.path().join("2")), Some(1111020202)).unwrap();
    import2_oids.push(crate::test::append_commit_from_objects(
        &import2, Some(1222020220), Some("readme for import2"), vec![
        Blob("README", dedent!("
        this is import2
        ")),
    ]).unwrap());
    import2_oids.push(crate::test::append_commit_from_objects(
        &import2, Some(1222020221), Some("adding import2"), vec![
        Blob("if2", dedent!("
        if2
        ")),
        Blob("README", dedent!("
        The readme for import2.
        ")),
    ]).unwrap());
    import2_oids.push(crate::test::append_commit_from_objects(
        &import2, Some(1222020222), Some("adding import1 as an import"), vec![
        Commit("import1", &format!("{}", import1_oids[1])),
        Blob(".gitmodules", dedent!(r#"
        [submodule "ext/import1"]
               path = import1
               url = http://models.example.com/w/import1
        "#)),
    ]).unwrap());

    // repodata
    let (_, repodata) = crate::test::repo_init(
        None, Some(&tempdir.path().join("3")), Some(1654321000)).unwrap();
    repodata_oids.push(crate::test::append_commit_from_objects(
        &repodata, Some(1666666700), Some("Initial commit of repodata"), vec![
        Blob("file1", dedent!("
        This is file1, initial commit.
        ")),
        Blob("README", dedent!("
        A simple readme file.
        ")),
    ]).unwrap());
    repodata_oids.push(crate::test::append_commit_from_objects(
        &repodata, Some(1666666710), Some("adding import1"), vec![
        Blob(".gitmodules", dedent!(r#"
        [submodule "ext/import1"]
               path = ext/import1
               url = http://models.example.com/w/import1
        "#)),
        Tree("ext", vec![
            Commit("import1", &format!("{}", import1_oids[0])),
        ]),
    ]).unwrap());
    repodata_oids.push(crate::test::append_commit_from_objects(
        &repodata, Some(1666666720), Some("adding some files"), vec![
        Tree("dir1", vec![
            Blob("file1", "file1 is new"),
            Blob("file2", "file2 is new"),
            Tree("nested", vec![
                Blob("file_a", "file_a is new"),
                Blob("file_b", "file_b is new"),
            ]),
        ]),
    ]).unwrap());
    repodata_oids.push(crate::test::append_commit_from_objects(
        &repodata, Some(1666666730), Some("bumping import1"), vec![
        Tree("ext", vec![
            Commit("import1", &format!("{}", import1_oids[1])),
        ]),
        Blob("file1", dedent!("
        This is file1, initial commit.
        This line added with import1 bump.
        ")),
        Blob("file2", dedent!("
        This is file2, added with import1 bump.
        ")),
    ]).unwrap());
    repodata_oids.push(crate::test::append_commit_from_objects(
        &repodata, Some(1666666740), Some("adding import2"), vec![
        Blob(".gitmodules", dedent!(r#"
        [submodule "ext/import1"]
               path = ext/import1
               url = http://models.example.com/w/import1
        [submodule "ext/import2"]
               path = ext/import2
               url = http://models.example.com/w/import2
        "#)),
        Tree("ext", vec![
            Commit("import2", &format!("{}", import2_oids[0])),
        ]),
    ]).unwrap());
    repodata_oids.push(crate::test::append_commit_from_objects(
        &repodata, Some(1666666750), Some("bumping import2"), vec![
        Tree("ext", vec![
            Commit("import2", &format!("{}", import2_oids[1])),
        ]),
    ]).unwrap());
    repodata_oids.push(crate::test::append_commit_from_objects(
        &repodata, Some(1666666760),
        Some("bumping import2, breaking import1"), vec![
        Tree("ext", vec![
            Commit("import1", &format!("{}", import2_oids[1])),
            Commit("import2", &format!("{}", import2_oids[2])),
        ]),
    ]).unwrap());
    repodata_oids.push(crate::test::append_commit_from_objects(
        &repodata, Some(1666666770),
        Some("fixing import1"), vec![
        Tree("ext", vec![
            Commit("import1", &format!("{}", import1_oids[1])),
        ]),
    ]).unwrap());
    repodata_oids.push(crate::test::append_commit_from_objects(
        &repodata, Some(1666666780), Some("updating dir1"), vec![
        Tree("dir1", vec![
            Blob("file2", "file2 is modified"),
            Tree("nested", vec![
                Blob("file_c", "file_c is new"),
            ]),
        ]),
    ]).unwrap());

    (
        tempdir,
        (import1, import1_oids),
        (import2, import2_oids),
        (repodata, repodata_oids),
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
