use async_recursion::async_recursion;
use futures::stream::StreamExt;
use futures::stream::futures_unordered::FuturesUnordered;
use gix::{
    Commit,
    Object,
    Repository,
    objs::{
        BlobRef,
        CommitRef,
        TreeRef,
        WriteTo,
        tree::EntryMode,
    },
    traverse::commit::Sorting,
    traverse::tree::Recorder,
};
use gix::object::Kind;
use pmrmodel_base::{
    repo::{
        TreeEntryInfo,
        LogEntryInfo,
        ObjectInfo,
        FileInfo,
        TreeInfo,
        CommitInfo,
        LogInfo,
        RemoteInfo,

        PathObject,
        PathInfo,
    },
    platform::Platform,
    workspace::{
        Workspace,
        WorkspaceSyncStatus,
        traits::{
            WorkspaceBackend,
            WorkspaceSyncBackend,
            WorkspaceTagBackend,
        },
    },
    merged::{
        WorkspacePathInfo,
    },
};

use pmrmodel::model::workspace_sync::{
    fail_sync,
};

use std::{
    io::Write,
    ops::Deref,
    path::{
        Path,
        PathBuf,
    },
};

use crate::{
    error::{
        ContentError,
        ExecutionError,
        GixError,
        PathError,
        PmrRepoError,
    },
    util::is_binary,
};

use super::util::*;
use super::*;

pub struct HandleW<'a, P: Platform> {
    platform: &'a P,
    git_root: PathBuf,
    pub workspace: &'a Workspace,
}

pub struct HandleWR<'a, P: Platform> {
    platform: &'a P,
    git_root: PathBuf,
    pub workspace: &'a Workspace,
    pub repo: Repository,
}

pub struct GitResult<'a> {
    pub repo: &'a Repository,
    pub commit: Commit<'a>,
    pub path: &'a str,
    pub target: GitResultTarget<'a>,
}

pub enum GitResultTarget<'a> {
    Object(Object<'a>),
    SubRepoPath {
        location: String,
        commit: String,
        path: &'a str,
    },
}

pub struct WorkspaceGitResult<'a>(&'a Workspace, &'a GitResult<'a>);

impl WorkspaceGitResult<'_> {
    pub fn new<'a>(
        workspace: &'a Workspace,
        git_result: &'a GitResult<'a>,
    ) -> WorkspaceGitResult<'a> {
        WorkspaceGitResult(&workspace, git_result)
    }
}

// These assume the blobs are all contained because the conversion to
// the Ref equivalent currently drops information for gix, and to make
// the internal usage consistent, the raw object is passed.
fn obj_blob_to_info(git_object: &Object, path: Option<&str>) -> ObjectInfo {
    let blob = BlobRef::from_bytes(&git_object.data).unwrap();
    ObjectInfo::FileInfo(FileInfo {
        size: blob.size() as u64,
        binary: is_binary(blob.data),
        mime_type: path
            .and_then(|path| mime_guess::from_path(path).first_raw())
            .unwrap_or("application/octet-stream")
            .to_string(),
    })
}

fn obj_tree_to_info(git_object: &Object) -> ObjectInfo {
    let tree = TreeRef::from_bytes(&git_object.data).unwrap();
    ObjectInfo::TreeInfo(
        TreeInfo {
            filecount: tree.entries.len() as u64,
            entries: tree.entries.iter().map(|entry| TreeEntryInfo {
                filemode: std::str::from_utf8(entry.mode.as_bytes()).unwrap().to_string(),
                kind: format!("{}", entry.oid.kind()),
                id: format!("{}", entry.oid),
                name: format!("{}", entry.filename),
            }).collect(),
        }
    )
}

fn obj_commit_to_info(git_object: &Object) -> ObjectInfo {
    ObjectInfo::CommitInfo(commitref_id_to_commitinfo(
        git_object.id.to_string(),
        CommitRef::from_bytes(&git_object.data)
            .expect("should have been verified as a well-formed commit"),
    ))
}

// practically duplicating the above.
fn commit_to_info(commit: &Commit) -> ObjectInfo {
    ObjectInfo::CommitInfo(commitref_id_to_commitinfo(
        commit.id.to_string(),
        CommitRef::from_bytes(&commit.data)
            .expect("should have been verified as a well-formed commit"),
    ))
}

fn commitref_id_to_commitinfo(
    commit_id: String,
    commit: CommitRef,
) -> CommitInfo {
    CommitInfo {
        commit_id: commit_id,
        author: format_signature_ref(&commit.author()),
        committer: format_signature_ref(&commit.committer()),
    }
}

impl From<&GitResult<'_>> for Option<PathObject> {
    fn from(git_result: &GitResult) -> Self {
        match &git_result.target {
            GitResultTarget::Object(object) => match gitresult_to_info(
                git_result,
                object,
            ) {
                Some(ObjectInfo::FileInfo(file_info)) => Some(PathObject::FileInfo(file_info)),
                Some(ObjectInfo::TreeInfo(tree_info)) => Some(PathObject::TreeInfo(tree_info)),
                _ => None,
            },
            GitResultTarget::SubRepoPath { location, commit, path } => {
                Some(PathObject::RemoteInfo(RemoteInfo {
                    location: location.to_string(),
                    commit: commit.to_string(),
                    path: path.to_string(),
                }))
            },
        }
    }
}

impl From<&GitResult<'_>> for PathInfo {
    fn from(git_result: &GitResult) -> Self {
        let commit_ref = CommitRef::from_bytes(&git_result.commit.data)
            .expect("should have been verified as a well-formed commit");
        PathInfo {
            commit: commitref_id_to_commitinfo(
                git_result.commit.id().to_string(),
                commit_ref,
            ),
            path: format!("{}", &git_result.path),
            object: git_result.into(),
        }
    }
}

impl From<&WorkspaceGitResult<'_>> for WorkspacePathInfo {
    fn from(
        WorkspaceGitResult(
            workspace,
            git_result,
        ): &WorkspaceGitResult<'_>
    ) -> Self {
        let commit_ref = CommitRef::from_bytes(&git_result.commit.data)
            .expect("should have been verified as a well-formed commit");
        WorkspacePathInfo {
            workspace_id: workspace.id,
            description: workspace.description.clone(),
            commit: commitref_id_to_commitinfo(
                git_result.commit.id().to_string(),
                commit_ref,
            ),
            path: format!("{}", &git_result.path),
            object: (*git_result).into(),
        }
    }
}

fn gitresult_to_info(git_result: &GitResult, git_object: &Object) -> Option<ObjectInfo> {
    // TODO split off to a formatter version?
    // alternatively, produce some structured data?
    match git_object.kind {
        Kind::Blob => {
            Some(obj_blob_to_info(
                &git_object,
                Some(git_result.path),
            ))
        },
        _ => object_to_info(git_object),
    }
}

fn object_to_info(
    git_object: &Object,
) -> Option<ObjectInfo> {
    match git_object.kind {
        Kind::Blob => {
            Some(obj_blob_to_info(
                &git_object,
                None,
            ))
        }
        Kind::Tree => {
            Some(obj_tree_to_info(&git_object))
        }
        Kind::Commit => {
            Some(obj_commit_to_info(&git_object))
        }
        Kind::Tag => {
            None
        }
    }
}

impl From<&GitResult<'_>> for Option<ObjectInfo> {
    fn from(git_result: &GitResult) -> Self {
        match &git_result.target {
            GitResultTarget::Object(object) => {
                object_to_info(&object)
            }
            _ => None
        }
    }
}

pub fn stream_git_result_default(mut writer: impl Write, git_result: &GitResult) -> std::result::Result<usize, std::io::Error> {
    // TODO split off to a formatter version?
    // alternatively, produce some structured data?
    writer.write(format!("
        have repo at {:?}
        have commit {:?}
        have commit_object {:?}
        using repopath {:?}
        have git_object {:?}
        have path_info {:?}
        \n",
        git_result.repo.path(),
        &git_result.commit.id(),
        commit_to_info(&git_result.commit),
        git_result.path,
        <Option<ObjectInfo>>::from(git_result),
        <PathInfo>::from(git_result),
    ).as_bytes())
}

pub fn stream_git_result_as_json(
    writer: impl Write,
    git_result: &GitResult,
) -> Result<(), serde_json::Error> {
    // TODO how to generalize this to deal with a common "theme" of JSON outputs?
    // currently, this is directly coupled to GitResult, but perhaps there needs
    // to be some trait that provide the output desired?
    // Also, need to consider how to provide a more generic JSON-LD builder framework
    // of sort?  Need to build context and what not...
    // generalize a UI based on that schema/grammar?
    serde_json::to_writer(writer, &<PathInfo>::from(git_result))
}

pub async fn stream_blob(
    mut writer: impl Write,
    blob: &Object<'_>,
) -> std::result::Result<usize, std::io::Error> {
    writer.write(&blob.data)
}

impl<'a, P: Platform> HandleW<'a, P> {
    pub fn new(
        platform: &'a P,
        git_root: PathBuf,
        workspace: &'a Workspace,
    ) -> Self {
        Self {
            platform: &platform,
            git_root: git_root,
            workspace: &workspace,
        }
    }

    pub async fn git_sync_workspace(self) -> Result<HandleWR<'a, P>, PmrRepoError> {
        // using libgit2 as mature protocol support is desired.
        let repo_dir = self.git_root.join(self.workspace.id.to_string());
        let repo_check = git2::Repository::open_bare(&repo_dir);

        info!("Syncing local {:?} with remote <{}>...", repo_dir, &self.workspace.url);
        let sync_id = WorkspaceSyncBackend::begin_sync(self.platform, self.workspace.id).await?;
        match repo_check {
            Ok(repo) => {
                info!("Found existing repo at {:?}, synchronizing...", repo_dir);
                let mut remote = repo.find_remote("origin")?;
                match remote.fetch(&[] as &[&str], None, None) {
                    Ok(_) => info!("Repository synchronized"),
                    Err(e) => {
                        fail_sync(self.platform, sync_id).await?;
                        return Err(ExecutionError::Synchronize {
                            workspace_id: self.workspace.id,
                            remote: self.workspace.url.clone(),
                            msg: e.to_string(),
                        }.into())
                    },
                };
            }
            Err(ref e) if e.class() == git2::ErrorClass::Repository => {
                fail_sync(self.platform, sync_id).await?;
                return Err(ExecutionError::Synchronize {
                    workspace_id: self.workspace.id,
                    remote: self.workspace.url.clone(),
                    msg: "expected local underlying repo to be a bare repo".to_string(),
                }.into())
            }
            Err(_) => {
                info!("Cloning new repository at {:?}...", repo_dir);
                let mut builder = git2::build::RepoBuilder::new();
                builder.bare(true);
                match builder.clone(&self.workspace.url, &repo_dir) {
                    Ok(_) => info!("Repository cloned"),
                    Err(e) => {
                        fail_sync(self.platform, sync_id).await?;
                        return Err(ExecutionError::Synchronize {
                            workspace_id: self.workspace.id,
                            remote: self.workspace.url.clone(),
                            msg: format!("fail to clone: {}", e),
                        }.into())
                    },
                };
            }
        }

        WorkspaceSyncBackend::complete_sync(self.platform, sync_id, WorkspaceSyncStatus::Completed).await?;
        let result = HandleWR::new(self.platform, self.git_root, &self.workspace)?;
        result.index_tags().await?;

        Ok(result)
    }
}


impl<'a, P: Platform> HandleWR<'a, P> {
    pub fn new(
        platform: &'a P,
        git_root: PathBuf,
        workspace: &'a Workspace,
    ) -> Result<Self, GixError> {
        let repo_dir = git_root.join(workspace.id.to_string());
        let repo = gix::open::Options::isolated()
            .open_path_as_is(true)
            .open(repo_dir)?
            .to_thread_local();
        Ok(Self {
            platform: &platform,
            git_root: git_root,
            workspace: &workspace,
            repo: repo,
        })
    }

    pub async fn index_tags(&self) -> Result<(), GixError> {
        let platform = self.platform;
        let workspace = &self.workspace;
        self.repo.references()?.tags()?
            .filter_map(|reference| {
                match reference {
                    Ok(tag) => {
                        let target = tag.target().id().to_hex().to_string();
                        match std::str::from_utf8(
                            tag.name().as_bstr().deref()
                        ) {
                            Ok(s) => Some((s.to_string(), target)),
                            Err(_) => {
                                warn!("\
                                a tag for commit_id {} omitted due to \
                                invalid utf8 encoding\
                                ", target
                                );
                                None
                            }
                        }
                    }
                    Err(e) => {
                        warn!("failed to decode a tag: {}", e);
                        None
                    }
                }
            })
            .map(|(name, oid)| async move {
                match WorkspaceTagBackend::index_workspace_tag(
                    platform,
                    workspace.id,
                    &name,
                    &oid,
                ).await {
                    Ok(_) => info!("indexed tag: {}", name),
                    Err(e) => warn!("tagging error: {:?}", e),
                }
            })
            .collect::<FuturesUnordered<_>>().collect::<Vec<_>>().await;

        Ok(())
    }

    pub async fn get_obj_by_spec(&self, spec: &str) -> Result<(), GixError> {
        let obj = self.repo.rev_parse_single(spec)?.object()?;
        info!("Found object {} {}", obj.kind, obj.id);
        info!("{:?}", object_to_info(&obj));
        Ok(())
    }

    fn get_commit(
        &'a self,
        commit_id: Option<&'a str>,
    ) -> Result<Commit<'a>, PmrRepoError> {
        let obj = rev_parse_single(&self.repo, &commit_id.unwrap_or("HEAD"))?;
        match obj.kind {
            kind if kind == Kind::Commit => {
                info!("Found {} {}", kind, obj.id);
            }
            _ => return Err(PathError::NoSuchCommit {
                workspace_id: self.workspace.id,
                oid: commit_id.unwrap_or("HEAD?").into(),
            }.into())
        }
        match obj.try_into_commit() {
            Ok(commit) => Ok(commit),
            Err(obj) => Err(ExecutionError::Unexpected {
                workspace_id: self.workspace.id,
                msg: format!("gix said oid {:?} was a commit?", obj.id),
            }.into()),
        }
    }

    // commit_id/path should be a pathinfo struct?
    pub fn pathinfo(
        &self,
        commit_id: Option<&'a str>,
        path: Option<&'a str>,
    ) -> Result<GitResult, PmrRepoError> {
        let commit = self.get_commit(commit_id)?;
        let tree = commit
            .tree_id().map_err(GixError::from)?
            .object().map_err(GixError::from)?;

        let (path, target) = match path {
            Some("") | Some("/") | None => {
                info!("No path provided; using root tree entry");
                ("".as_ref(), GitResultTarget::Object(tree))
            },
            Some(s) => {
                let path = s.strip_prefix('/').unwrap_or(&s);
                let mut comps = Path::new(path).components();
                let mut curr_path = PathBuf::new();
                let mut object = Some(tree);
                let mut target: Option<GitResultTarget> = None;

                while let Some(component) = comps.next() {
                    let entry = object
                        .expect("iteration has this set or look breaked")
                        .try_into_tree().map_err(GixError::from)?
                        .lookup_entry_by_path(
                            Path::new(&component)
                        ).map_err(GixError::from)?
                        .ok_or_else(
                            || PmrRepoError::from(PathError::NoSuchPath {
                                workspace_id: self.workspace.id,
                                oid: commit.id.to_string(),
                                path: path.to_string(),
                            })
                        )?;
                    curr_path.push(component);
                    match entry.mode() {
                        EntryMode::Commit => {
                            info!("entry {:?} is a commit", entry.id());
                            let location = get_submodule_target(
                                &commit,
                                self.workspace.id,
                                curr_path.to_str().unwrap(),
                            )?;
                            target = Some(GitResultTarget::SubRepoPath {
                                location: location,
                                commit: entry.id().to_string(),
                                path: comps.as_path().to_str().unwrap(),
                            });
                            object = None;
                            break;
                        }
                        _ => ()
                    }
                    let next_object = entry
                        .object().map_err(GixError::from)?;
                    info!("got {} {:?}", next_object.kind, &next_object);
                    object = Some(next_object);
                };
                match object {
                    Some(object) =>
                        (path, GitResultTarget::Object(object)),
                    None =>
                        // Only way object is None is have target set.
                        (path, target.expect("to be a SubRepoPath")),
                }
            },
        };
        let git_result = GitResult {
            repo: &self.repo,
            commit: commit,
            path: path,
            target: target,
        };
        Ok(git_result)
    }

    #[async_recursion(?Send)]
    pub async fn stream_result_blob(
        &self,
        writer: impl Write + 'async_recursion,
        git_result: &GitResult<'a>,
    ) -> Result<usize, PmrRepoError> {
        match &git_result.target {
            GitResultTarget::Object(object) => match object.kind {
                Kind::Blob => Ok(stream_blob(writer, object).await?),
                _ => Err(ContentError::Invalid {
                    workspace_id: self.workspace.id,
                    oid: git_result.commit.id().to_string(),
                    path: git_result.path.to_string(),
                    msg: format!("expected to be a blob"),
                }.into())
            },
            GitResultTarget::SubRepoPath { location, commit, path } => {
                let workspaces = WorkspaceBackend::list_workspace_by_url(
                    self.platform, &location,
                ).await?;
                if workspaces.len() == 0 {
                    return Err(ContentError::NoWorkspaceForUrl{
                        workspace_id: self.workspace.id,
                        url: location.to_string(),
                    }.into())
                }
                // TODO need to derive this for this specific workspace
                // for now, just use the first result.
                let pmrbackend = HandleWR::new(
                    self.platform, self.git_root.clone(), &workspaces[0])?;
                let git_result = pmrbackend.pathinfo(
                    Some(commit), Some(path),
                )?;
                Ok(self.stream_result_blob(writer, &git_result).await?)
            },
        }
    }

    pub fn loginfo(
        &self,
        commit_id: Option<&str>,
        path: Option<&'a str>,
        count: Option<usize>,
    ) -> Result<LogInfo, PmrRepoError> {
        let commit = self.get_commit(commit_id)?;
        let mut filter = PathFilter::new(&self.repo, path);
        let log_entry_iter = self.repo
            .rev_walk([commit.id])
            .sorting(Sorting::ByCommitTimeNewestFirst)
            .all().map_err(GixError::from)?
            .filter(|info| info.as_ref()
                .map(|info| filter.check(info))
                .unwrap_or(true)
            )
            .map(|info| {
                let commit = info?.object()?;
                let commit_ref = CommitRef::from_bytes(&commit.data)?;
                let committer = commit_ref.committer;
                Ok(LogEntryInfo {
                    commit_id: format!("{}", commit.id()),
                    author: format_signature_ref(&commit_ref.author),
                    committer: format_signature_ref(&committer),
                    commit_timestamp: committer.time.seconds,
                    message: commit_ref.message.to_string(),
                })
            });

        let log_entries = match count {
            Some(count) => log_entry_iter
                .take(count)
                .collect::<Result<Vec<_>, GixError>>()?,
            None => log_entry_iter
                .collect::<Result<Vec<_>, GixError>>()?,
        };

        Ok(LogInfo { entries: log_entries })
    }

    pub fn files(
        &self,
        commit_id: Option<&str>,
    ) -> Result<Vec<String>, PmrRepoError> {
        let commit = self.get_commit(commit_id)?;
        let tree = commit.tree().map_err(GixError::from)?;
        let mut recorder = Recorder::default();
        tree.traverse()
            .breadthfirst(&mut recorder).map_err(GixError::from)?;
        let mut results = recorder.records.iter()
            .filter(|entry| entry.mode != EntryMode::Tree)
            .filter_map(
                |entry| std::str::from_utf8(entry.filepath.as_ref()).ok()
            )
            .map(|x| x.to_string())
            .collect::<Vec<_>>();
        results.sort();
        Ok(results)
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    use mockall::predicate::*;
    use tempfile::TempDir;
    use crate::test::MockPlatform as MockBackend;

    // helper to deal with moves of the workspace record.
    async fn git_sync_helper(
        mock_backend: &MockBackend, id: i64, url: &str, git_root: &TempDir
    ) -> Result<(), PmrRepoError> {
        let workspace = Workspace {
            id: id,
            superceded_by_id: None,
            url: url.to_string(),
            description: None,
            long_description: None,
            created_ts: 1234567890,
            exposures: None,
        };
        let pmrbackend = HandleW::new(mock_backend, git_root.path().to_owned().to_path_buf(), &workspace);
        pmrbackend.git_sync_workspace().await?;
        Ok(())
    }

    #[async_std::test]
    async fn test_git_sync_workspace_empty() {
        let (td_, _) = crate::test::repo_init(None, None, None).unwrap();
        let td = td_.unwrap();
        let mut mock_backend = MockBackend::new();
        mock_backend.expect_begin_sync()
            .times(1)
            .with(eq(0))
            .returning(|_| Ok(1));
        mock_backend.expect_complete_sync()
            .times(1)
            .with(eq(1), eq(WorkspaceSyncStatus::Completed))
            .returning(|_, _| Ok(true));
        let git_root = TempDir::new().unwrap();
        git_sync_helper(&mock_backend, 0, td.path().to_str().unwrap(), &git_root).await.unwrap();
    }

    #[async_std::test]
    async fn test_git_sync_workspace_with_index_tag() {
        let (td_, _) = crate::test::repo_init(None, None, None).unwrap();
        let td = td_.as_ref().unwrap();
        // TODO use gix to tag?
        let repo = git2::Repository::open_bare(td).unwrap();
        let id = repo.head().unwrap().target().unwrap();
        let obj = repo.find_object(id, None).unwrap();
        repo.tag_lightweight("new_tag", &obj, false).unwrap();

        let mut mock_backend = MockBackend::new();
        mock_backend.expect_begin_sync()
            .times(1)
            .with(eq(123))
            .returning(|_| Ok(1));
        mock_backend.expect_complete_sync()
            .times(1)
            .with(eq(1), eq(WorkspaceSyncStatus::Completed))
            .returning(|_, _| Ok(true));
        mock_backend.expect_index_workspace_tag()
            .times(1)
            .withf(move |workspace_id: &i64, name: &str, commit_id: &str| {
                *workspace_id == 123 && name == "refs/tags/new_tag" && commit_id == format!("{}", id)
            })
            .returning(|_, _, _| Ok(1));

        let git_root = TempDir::new().unwrap();
        git_sync_helper(&mock_backend, 123, td.path().to_str().unwrap(), &git_root).await.unwrap();
    }

    #[async_std::test]
    async fn test_git_sync_failure_invalid_remote() {
        // where remote couldn't be found or invalid.
        let td = TempDir::new().unwrap();
        let err_msg = format!(
            "ExecutionError: workspace `{0}`: failed to synchronize with remote `{1}`: \
            fail to clone: could not find repository from '{1}'; \
            class=Repository (6); code=NotFound (-3)", 2, td.path().to_str().unwrap());
        let mut mock_backend = MockBackend::new();
        mock_backend.expect_begin_sync()
            .times(1)
            .with(eq(2))
            .returning(|_| Ok(3));
        mock_backend.expect_complete_sync()
            .times(1)
            .with(eq(3), eq(WorkspaceSyncStatus::Error))
            .returning(|_, _| Ok(true));

        let git_root = TempDir::new().unwrap();
        let err = git_sync_helper(
            &mock_backend, 2, td.path().to_str().unwrap(), &git_root).await.unwrap_err();
        assert_eq!(err.to_string(), err_msg);
    }

    #[async_std::test]
    async fn test_git_sync_failure_dropped_source() {
        let (td_, _) = crate::test::repo_init(None, None, None).unwrap();
        let td = td_.unwrap();
        let mut mock_backend = MockBackend::new();
        mock_backend.expect_begin_sync()
            .times(1)
            .with(eq(42))
            .returning(|_| Ok(1));
        mock_backend.expect_complete_sync()
            .times(1)
            .with(eq(1), eq(WorkspaceSyncStatus::Completed))
            .returning(|_, _| Ok(true));

        let git_root = TempDir::new().unwrap();

        let td_path = td.path().to_owned();
        let url = td_path.to_str().unwrap();
        assert!(git_sync_helper(&mock_backend, 42, url, &git_root).await.is_ok());

        td.close().unwrap();
        mock_backend.checkpoint();

        // now verify that the failure to sync will generate the right error
        // when an originally working remote disappeared or errored.
        mock_backend.expect_begin_sync()
            .times(1)
            .with(eq(42))
            .returning(|_| Ok(2));
        mock_backend.expect_complete_sync()
            .times(1)
            .with(eq(2), eq(WorkspaceSyncStatus::Error))
            .returning(|_, _| Ok(true));

        let failed_sync = git_sync_helper(&mock_backend, 42, url, &git_root).await;
        let err_msg = format!(
            "ExecutionError: workspace `42`: failed to synchronize with \
            remote `{}`: unsupported URL protocol; class=Net (12)", url
        );
        assert_eq!(failed_sync.unwrap_err().to_string(), err_msg);
    }

    #[async_std::test]
    async fn test_git_sync_workspace_not_bare() {
        let (origin_, _) = crate::test::repo_init(None, None, None).unwrap();
        let origin = origin_.unwrap();

        let git_root_dir = TempDir::new().unwrap();
        let repo_dir = git_root_dir.path().join("10");

        let mut repo = gix::ThreadSafeRepository::init_opts(
            &repo_dir,
            gix::create::Kind::WithWorktree,
            gix::create::Options::default(),
            gix::open::Options::isolated(),
        ).unwrap().to_thread_local();
        let mut config = repo.config_snapshot_mut();
        config.set_raw_value("committer", None, "name", "user").unwrap();
        config.set_raw_value("committer", None, "email", "user@example.com").unwrap();
        drop(config);
        crate::test::init_empty_commit(&repo, None).unwrap();
        crate::test::commit(&repo, vec![("some_file", "")]).unwrap();

        let mut mock_backend = MockBackend::new();
        mock_backend.expect_begin_sync()
            .times(1)
            .with(eq(10))
            .returning(|_| Ok(1));
        mock_backend.expect_complete_sync()
            .times(1)
            .with(eq(1), eq(WorkspaceSyncStatus::Error))
            .returning(|_, _| Ok(true));
        let failed_sync = git_sync_helper(
            &mock_backend, 10, origin.path().to_str().unwrap(), &git_root_dir
        ).await.unwrap_err();
        let err_msg = format!(
            "ExecutionError: workspace `10`: failed to synchronize with \
            remote `{}`: expected local underlying repo to be a bare repo",
            origin.path().display(),
        );
        assert_eq!(failed_sync.to_string(), err_msg);
    }

    #[async_std::test]
    async fn test_workspace_path_info_from_workspace_git_result() {
        let (td_, repo) = crate::test::repo_init(None, None, None).unwrap();
        crate::test::commit(&repo, vec![("some_file", "")]).unwrap();

        let td = td_.unwrap();
        let td_path = td.path().to_owned();
        let url = td_path.to_str().unwrap();

        let git_root = TempDir::new().unwrap();
        let mut mock_backend = MockBackend::new();
        mock_backend.expect_begin_sync()
            .times(1)
            .with(eq(10))
            .returning(|_| Ok(10));
        mock_backend.expect_complete_sync()
            .times(1)
            .with(eq(10), eq(WorkspaceSyncStatus::Completed))
            .returning(|_, _| Ok(true));
        assert!(git_sync_helper(&mock_backend, 10, url, &git_root).await.is_ok());

        let workspace = Workspace {
            id: 10,
            url: "http://example.com/10".to_string(),
            superceded_by_id: None,
            description: Some("demo workspace 10".to_string()),
            long_description: None,
            created_ts: 1234567890,
            exposures: None,
        };

        let pmrbackend = HandleWR::new(
            &mock_backend,
            git_root.path().to_path_buf(),
            &workspace,
        ).unwrap();

        let result = pmrbackend.pathinfo(None, None).unwrap();
        let pathinfo = <WorkspacePathInfo>::from(&WorkspaceGitResult::new(&pmrbackend.workspace, &result));
        assert_eq!(pathinfo.path, "".to_string());
        assert_eq!(pathinfo.description, Some("demo workspace 10".to_string()));
    }

    #[async_std::test]
    async fn test_workspace_submodule_access() {
        let (
            git_root,
            _, // (import1, import1_oids),
            _, // (import2, import2_oids),
            _, // (repodata, repodata_oids)
        ) = crate::test::create_repodata();

        // let import1_workspace = Workspace {
        //     id: 1,
        //     url: "http://models.example.com/w/import1".to_string(),
        //     description: Some("The import1 workspace".to_string())
        // };
        // let import2_workspace = Workspace {
        //     id: 2,
        //     url: "http://models.example.com/w/import2".to_string(),
        //     description: Some("The import2 workspace".to_string())
        // };
        let repodata_workspace = Workspace {
            id: 3,
            url: "http://models.example.com/w/repodata".to_string(),
            description: Some("The repodata workspace".to_string()),
            superceded_by_id: None,
            long_description: None,
            created_ts: 1234567890,
            exposures: None,
        };

        let mut mock_backend = MockBackend::new();
        // used later.
        mock_backend.expect_list_workspace_by_url()
            .times(1)
            .with(eq("http://models.example.com/w/import2"))
            .returning(|_| Ok([Workspace {
                id: 2,
                url: "http://models.example.com/w/import2".to_string(),
                description: Some("The import2 workspace".to_string()),
                superceded_by_id: None,
                long_description: None,
                created_ts: 1234567890,
                exposures: None,
            }].into()));
        let pmrbackend = HandleWR::new(
            &mock_backend,
            git_root.path().to_path_buf(),
            &repodata_workspace,
        ).unwrap();

        let result = pmrbackend.pathinfo(
            Some("557ee3cb13fb421d2bd6897615ae95830eb427c8"),
            Some("ext/import1/README"),
        ).unwrap();

        let pathinfo = <WorkspacePathInfo>::from(&WorkspaceGitResult::new(
            &pmrbackend.workspace, &result));
        assert_eq!(
            pathinfo.path,
            "ext/import1/README".to_string());
        assert_eq!(
            pathinfo.object,
            Some(PathObject::RemoteInfo(RemoteInfo {
                location: "http://models.example.com/w/import1"
                    .to_string(),
                commit: "01b952d14a0a33d22a0aa465fe763e5d17b15d46"
                    .to_string(),
                path: "README".to_string(),
            })),
        );

        let result = pmrbackend.pathinfo(
            Some("c4d735e5a305559c1cb0ce8de4c25ed5c3f4f263"),
            Some("ext/import2/import1/if1"),
        ).unwrap();
        let pathinfo = <WorkspacePathInfo>::from(&WorkspaceGitResult::new(
            &pmrbackend.workspace, &result));
        assert_eq!(
            pathinfo.path,
            "ext/import2/import1/if1".to_string());
        assert_eq!(
            pathinfo.object,
            Some(PathObject::RemoteInfo(RemoteInfo {
                location: "http://models.example.com/w/import2"
                    .to_string(),
                commit: "0ab8a26a0e85a033bea0388216667d83cc0dc1dd"
                    .to_string(),
                path: "import1/if1".to_string(),
            })),
        );

        let mut buffer = <Vec<u8>>::new();
        let readme_result = pmrbackend.pathinfo(
            Some("557ee3cb13fb421d2bd6897615ae95830eb427c8"),
            Some("README"),
        ).unwrap();
        assert_eq!(
            pmrbackend.stream_result_blob(&mut buffer, &readme_result).await.unwrap(),
            22,
        );
        assert_eq!(
            std::str::from_utf8(&buffer).unwrap(),
            "A simple readme file.\n",
        );

        let mut buffer = <Vec<u8>>::new();
        let import2_result = pmrbackend.pathinfo(
            Some("a4a04eed5e243e3019592579a7f6eb950399f9bf"),
            Some("ext/import2/if2"),
        ).unwrap();
        assert_eq!(
            pmrbackend.stream_result_blob(&mut buffer, &import2_result).await.unwrap(),
            4,
        );
        assert_eq!(
            std::str::from_utf8(&buffer).unwrap(),
            "if2\n",
        );

    }

    #[test]
    fn test_workspace_loginfo() -> anyhow::Result<()> {
        let (
            git_root,
            _, // (import1, import1_oids),
            _, // (import2, import2_oids),
            (_, repodata_oids),
        ) = crate::test::create_repodata();
        let repodata_workspace = Workspace {
            id: 3,
            url: "http://models.example.com/w/repodata".to_string(),
            description: Some("The repodata workspace".to_string()),
            superceded_by_id: None,
            long_description: None,
            created_ts: 1234567890,
            exposures: None,
        };
        let mock_backend = MockBackend::new();
        let pmrbackend = HandleWR::new(
            &mock_backend,
            git_root.path().to_path_buf(),
            &repodata_workspace,
        ).unwrap();
        let logs = pmrbackend.loginfo(None, None, None).unwrap();
        assert_eq!(
            repodata_oids.iter()
                .map(|x| x.to_string())
                .collect::<Vec<_>>(),
            logs.entries.iter()
                .map(|x| x.commit_id.clone())
                .rev()
                .skip(1)  // skip the initial commit
                .collect::<Vec<_>>(),
        );
        assert_eq!(logs, serde_json::from_str(r#"{
          "entries": [
            {
              "commit_id": "8ae6e9af37c8bd78614545d0ab807348fc46dcab",
              "author": "user <user@example.com>",
              "committer": "user <user@example.com>",
              "commit_timestamp": 1666666780,
              "message": "updating dir1"
            },
            {
              "commit_id": "c4d735e5a305559c1cb0ce8de4c25ed5c3f4f263",
              "author": "user <user@example.com>",
              "committer": "user <user@example.com>",
              "commit_timestamp": 1666666770,
              "message": "fixing import1"
            },
            {
              "commit_id": "a4a04eed5e243e3019592579a7f6eb950399f9bf",
              "author": "user <user@example.com>",
              "committer": "user <user@example.com>",
              "commit_timestamp": 1666666760,
              "message": "bumping import2, breaking import1"
            },
            {
              "commit_id": "502b18ac456c8e475f731cbfe568fd6eb1177327",
              "author": "user <user@example.com>",
              "committer": "user <user@example.com>",
              "commit_timestamp": 1666666750,
              "message": "bumping import2"
            },
            {
              "commit_id": "965ccc1276832489c69b680b49874a6e1dc1743b",
              "author": "user <user@example.com>",
              "committer": "user <user@example.com>",
              "commit_timestamp": 1666666740,
              "message": "adding import2"
            },
            {
              "commit_id": "27be7efbe5fcccda5ee6ca00ef96834f592139a5",
              "author": "user <user@example.com>",
              "committer": "user <user@example.com>",
              "commit_timestamp": 1666666730,
              "message": "bumping import1"
            },
            {
              "commit_id": "e931905807563cb5353958e865d72fed12dccd4f",
              "author": "user <user@example.com>",
              "committer": "user <user@example.com>",
              "commit_timestamp": 1666666720,
              "message": "adding some files"
            },
            {
              "commit_id": "557ee3cb13fb421d2bd6897615ae95830eb427c8",
              "author": "user <user@example.com>",
              "committer": "user <user@example.com>",
              "commit_timestamp": 1666666710,
              "message": "adding import1"
            },
            {
              "commit_id": "9f02f69509110e7235e4bb9f50e235a246ae9f5c",
              "author": "user <user@example.com>",
              "committer": "user <user@example.com>",
              "commit_timestamp": 1666666700,
              "message": "Initial commit of repodata"
            },
            {
              "commit_id": "e55a6e1058fe4caf81e5cdfe806a3f86e1b94fb2",
              "author": "user <user@example.com>",
              "committer": "user <user@example.com>",
              "commit_timestamp": 1654321000,
              "message": "initial commit"
            }
          ]
        }"#)?);

        // for a specific path
        assert_eq!(
            [
                "27be7efbe5fcccda5ee6ca00ef96834f592139a5",
                "9f02f69509110e7235e4bb9f50e235a246ae9f5c",
            ],
            pmrbackend
                .loginfo(None, Some("file1"), None)?
                .entries.iter()
                .map(|x| x.commit_id.to_string())
                .collect::<Vec<_>>()
                .as_ref(),
        );
        assert_eq!(
            [
                "c4d735e5a305559c1cb0ce8de4c25ed5c3f4f263",
                "a4a04eed5e243e3019592579a7f6eb950399f9bf",
                "27be7efbe5fcccda5ee6ca00ef96834f592139a5",
                "557ee3cb13fb421d2bd6897615ae95830eb427c8",
            ],
            pmrbackend
                .loginfo(None, Some("ext/import1"), None)?
                .entries.iter()
                .map(|x| x.commit_id.to_string())
                .collect::<Vec<_>>()
                .as_ref(),
        );
        assert_eq!(
            0,
            pmrbackend
                .loginfo(None, Some("no/such/path"), None)?
                .entries.iter()
                .map(|x| x.commit_id.to_string())
                .collect::<Vec<_>>()
                .len(),
        );

        // from both a path and commit
        assert_eq!(
            [
                "27be7efbe5fcccda5ee6ca00ef96834f592139a5",
                "557ee3cb13fb421d2bd6897615ae95830eb427c8",
            ],
            pmrbackend
                .loginfo(
                    Some("502b18ac456c8e475f731cbfe568fd6eb1177327"),
                    Some("ext/import1"),
                    None,
                )?
                .entries.iter()
                .map(|x| x.commit_id.to_string())
                .collect::<Vec<_>>()
                .as_ref(),
        );

        // from both a path and commit and count
        assert_eq!(
            [
                "27be7efbe5fcccda5ee6ca00ef96834f592139a5",
            ],
            pmrbackend
                .loginfo(
                    Some("502b18ac456c8e475f731cbfe568fd6eb1177327"),
                    Some("ext/import1"),
                    Some(1),
                )?
                .entries.iter()
                .map(|x| x.commit_id.to_string())
                .collect::<Vec<_>>()
                .as_ref(),
        );

        Ok(())
    }

    #[test]
    fn test_workspace_files() -> anyhow::Result<()> {
        let (
            git_root,
            _, // (import1, import1_oids),
            _, // (import2, import2_oids),
            _, // (repodata, repodata_oids)
        ) = crate::test::create_repodata();
        let repodata_workspace = Workspace {
            id: 3,
            url: "http://models.example.com/w/repodata".to_string(),
            description: Some("The repodata workspace".to_string()),
            superceded_by_id: None,
            long_description: None,
            created_ts: 1234567890,
            exposures: None,
        };
        let mock_backend = MockBackend::new();
        let pmrbackend = HandleWR::new(
            &mock_backend,
            git_root.path().to_path_buf(),
            &repodata_workspace,
        )?;

        assert_eq!(
            pmrbackend.files(None)?,
            [
                ".gitmodules",
                "README",
                "dir1/file1",
                "dir1/file2",
                "dir1/nested/file_a",
                "dir1/nested/file_b",
                "dir1/nested/file_c",
                "ext/import1",
                "ext/import2",
                "file1",
                "file2",
            ]
        );
        assert_eq!(
            pmrbackend.files(
                Some("9f02f69509110e7235e4bb9f50e235a246ae9f5c"))?,
            [
                "README",
                "file1",
            ]
        );

        Ok(())
    }

}
