use async_recursion::async_recursion;
use futures::stream::{
    StreamExt as _,
    futures_unordered::FuturesUnordered,
};
use gix::{
    object::Kind,
    objs::{
        CommitRef,
        tree::EntryKind,
    },
    traverse::commit::Sorting,
    Commit,
    Repository,
};
use pmrcore::{
    git::PathObjectDetached,
    repo::{
        LogEntryInfo,
        LogInfo,
        PathObjectInfo,
        RemoteInfo,
        RepoResult,
    },
    platform::MCPlatform,
    workspace::{
        WorkspaceRef,
        traits::{
            Workspace as _,
            WorkspaceBackend,
            WorkspaceTagBackend,
        },
    },
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
    backend::Backend,
    error::{
        ContentError,
        GixError,
        PathError,
        PmrRepoError,
    },
};

use super::{
    Handle,
    GitHandle,
    GitHandleResult,
    GitResultTarget,
    util::*,
};

impl<P: MCPlatform + Sync> From<&GitHandleResult<'_, '_, P>> for PathObjectInfo {
    fn from(item: &GitHandleResult<P>) -> Self {
        match &item.target {
            GitResultTarget::Object(object) => object.into(),
            GitResultTarget::RemoteInfo(remote_info) => PathObjectInfo::RemoteInfo(remote_info.clone()),
        }
    }
}

impl<P: MCPlatform + Sync> From<GitHandleResult<'_, '_, P>> for RepoResult {
    fn from(item: GitHandleResult<P>) -> Self {
        RepoResult {
            target: (&item).into(),
            workspace: item.workspace.clone_inner(),
            path: item.path().to_string(),
            commit: item.commit.try_into()
                .expect("commit should have been parsed during processing"),
        }
    }
}

impl<'handle, 'db, P: MCPlatform + Sync> TryFrom<Handle<'handle, 'db, P>> for GitHandle<'handle, 'db, P> {
    type Error = GixError;

    fn try_from(item: Handle<'handle, 'db, P>) -> Result<Self, GixError> {
        let repo = gix::open::Options::isolated()
            .open_path_as_is(true)
            .open(&item.repo_dir)?
            .to_thread_local();
        Ok(Self {
            backend: item.backend,
            workspace: item.workspace,
            repo
        })
    }
}

impl<'db, 'repo, P: MCPlatform + Sync> GitHandle<'db, 'repo, P> {
    pub(crate) fn new(
        backend: &'repo Backend<'db, P>,
        repo_root: PathBuf,
        workspace: WorkspaceRef<'db, P>,
    ) -> Result<Self, GixError> {
        let repo_dir = repo_root.join(workspace.id().to_string());
        let repo = gix::open::Options::isolated()
            .open_path_as_is(true)
            .open(&repo_dir)?
            .to_thread_local();
        Ok(Self { backend, workspace, repo })
    }

    pub fn workspace(&'repo self) -> &'repo WorkspaceRef<'db, P> {
        &self.workspace
    }

    pub fn repo(&'repo self) -> &'repo Repository {
        &self.repo
    }

    pub async fn index_tags(&self) -> Result<(), GixError> {
        let platform = self.backend.db_platform;
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
                    workspace.id(),
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

    // commit_id/path should be a pathinfo struct?
    pub fn pathinfo<S: AsRef<str>>(
        &'repo self,
        commit_id: Option<&'repo str>,
        path: Option<S>,
    ) -> Result<GitHandleResult<'db, 'repo, P>, PmrRepoError> {
        let workspace_id = self.workspace.id();
        let commit = get_commit(&self.repo, workspace_id, commit_id)?;
        let tree = commit
            .tree_id().map_err(GixError::from)?
            .object().map_err(GixError::from)?;

        let target = match path.as_ref().map(|s| s.as_ref()) {
            Some("") | Some("/") | None => {
                info!("No path provided; using root tree entry");
                GitResultTarget::Object(
                    PathObjectDetached::new("".to_string(), tree.into()),
                )
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
                        .peel_to_entry_by_path(
                            Path::new(&component)
                        ).map_err(GixError::from)?
                        .ok_or_else(
                            || PmrRepoError::from(PathError::NoSuchPath {
                                workspace_id: workspace_id,
                                oid: commit.id.to_string(),
                                path: path.to_string(),
                            })
                        )?;
                    curr_path.push(component);
                    match entry.mode() {
                        k if (k == EntryKind::Commit.into()) => {
                            info!("entry {:?} is a commit", entry.id());
                            let location = get_submodule_target(
                                &commit,
                                workspace_id,
                                curr_path.to_str().unwrap(),
                            )?;
                            target = Some(GitResultTarget::RemoteInfo(RemoteInfo {
                                location: location,
                                commit: entry.id().to_string(),
                                subpath: comps.as_path().to_str().unwrap().to_string(),
                                path: path.to_string(),
                            }));
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
                        GitResultTarget::Object(
                            PathObjectDetached::new(path.to_string(), object.into())
                        ),
                    None =>
                        // Only way object is None is have target set.
                        target.expect("to be a RemoteInfo"),
                }
            },
        };
        let item = GitHandleResult {
            backend: &self.backend,
            repo: &self.repo,
            commit: commit.into(),
            target: target,
            workspace: &self.workspace,
        };
        Ok(item)
    }

    pub fn loginfo(
        &self,
        commit_id: Option<&str>,
        path: Option<&'repo str>,
        count: Option<usize>,
    ) -> Result<LogInfo, PmrRepoError> {
        let workspace_id = self.workspace.id();
        let commit = get_commit(&self.repo, workspace_id, commit_id)?;
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
        let workspace_id = self.workspace.id();
        let commit = get_commit(&self.repo, workspace_id, commit_id)?;
        files(&commit)
    }

}

impl<'db, 'repo, P: MCPlatform + Sync> GitHandleResult<'db, 'repo, P> {
    pub fn repo(&'repo self) -> &'repo Repository {
        self.repo
    }

    pub fn commit(&'repo self) -> Commit<'repo> {
        self.commit.clone().attach(self.repo).into_commit()
    }

    pub fn path(&self) -> &str {
        match &self.target {
            GitResultTarget::Object(object) => &object.path,
            GitResultTarget::RemoteInfo(remote_info) => &remote_info.path,
        }
    }

    // TODO could use an TryInto<PathObject<'repo>> or something along that line
    // for getting the final result.
    pub fn target(&'repo self) -> &GitResultTarget {
        &self.target
    }

    pub fn workspace(&'db self) -> &WorkspaceRef<'db, P> {
        &self.workspace
    }

    #[async_recursion(?Send)]
    pub async fn stream_blob(
        &self,
        mut writer: impl Write + 'async_recursion,
    ) -> Result<usize, PmrRepoError> {
        match &self.target {
            GitResultTarget::Object(object) => match object.object.kind {
                Kind::Blob => Ok(writer.write(&object.object.data)?),
                _ => Err(ContentError::Invalid {
                    workspace_id: self.workspace.id(),
                    oid: self.commit.id.to_string(),
                    path: self.path().to_string(),
                    msg: format!("expected to be a blob"),
                }.into())
            },
            GitResultTarget::RemoteInfo(RemoteInfo { location, commit, subpath, .. }) => {
                let workspaces = WorkspaceBackend::list_workspace_by_url(
                    self.backend.db_platform, &location,
                ).await?;
                if workspaces.len() == 0 {
                    return Err(ContentError::NoWorkspaceForUrl{
                        workspace_id: self.workspace.id(),
                        url: location.to_string(),
                    }.into())
                }
                // TODO need to derive this for this specific workspace
                // for now, just use the first result.
                // TODO figure out how to acquire the git_handle using the url
                // instead?
                let handle = self.backend.git_handle(workspaces[0].id).await?;
                let git_result = handle.pathinfo(Some(&commit), Some(&subpath))?;
                git_result.stream_blob(writer).await
            },
        }
    }

    /// Return the list of files associated with the commit that this
    /// `GitHandleResult` is associated with.
    pub fn files(
        &self,
    ) -> Result<Vec<String>, PmrRepoError> {
        files(&self.commit())
    }
}

