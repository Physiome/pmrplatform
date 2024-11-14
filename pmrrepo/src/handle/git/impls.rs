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
    sync::OnceLock,
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

impl From<&GitHandleResult<'_>> for Option<PathObjectInfo> {
    fn from(item: &GitHandleResult) -> Self {
        item.target
            .as_ref()
            .map(|target| match target {
                GitResultTarget::Object(object) => object.into(),
                GitResultTarget::RemoteInfo(remote_info) => PathObjectInfo::RemoteInfo(remote_info.clone()),
            })
    }
}

impl From<GitHandleResult<'_>> for RepoResult {
    fn from(item: GitHandleResult) -> Self {
        RepoResult {
            target: (&item).into(),
            workspace: item.workspace.clone_inner(),
            path: item.path().map(str::to_string),
            commit: item.commit
                .map(|commit| commit.try_into()
                    .expect("commit should have been parsed during processing")),
        }
    }
}

impl<'handle> From<Handle<'handle>> for GitHandle<'handle> {
    fn from(item: Handle<'handle>) -> Self {
        let repo = OnceLock::new();
        Self {
            backend: item.backend,
            workspace: item.workspace,
            repo_dir: item.repo_dir,
            repo,
        }
    }
}

impl<'repo> GitHandle<'repo> {
    pub(crate) fn new(
        backend: &'repo Backend,
        repo_root: PathBuf,
        workspace: WorkspaceRef<'repo>,
    ) -> Self {
        let repo_dir = repo_root.join(workspace.id().to_string());
        let repo = OnceLock::new();
        Self { backend, workspace, repo_dir, repo }
    }

    pub fn workspace(&self) -> &WorkspaceRef<'repo> {
        &self.workspace
    }

    pub fn repo(&self) -> Result<Repository, PathError> {
        self.repo.get_or_init(|| Ok(
            gix::open::Options::isolated()
                .open_path_as_is(true)
                .open(&self.repo_dir)?)
        )
            .as_ref()
            .map(|repo| repo.to_thread_local())
            .map_err(|_| PathError::Repository {
                workspace_id: self.workspace.id(),
            })
    }

    fn repo_tags(&self) -> Result<Vec<(String, String)>, PmrRepoError> {
        Ok(self.repo()?
            .references()
            .map_err(GixError::from)?
            .tags()
            .map_err(GixError::from)?
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
            .collect::<Vec<_>>())
    }

    pub async fn index_tags(&self) -> Result<(), PmrRepoError> {
        let workspace = &self.workspace;
        self.repo_tags()?
            .into_iter()
            .map(|(name, oid)| async move {
                match WorkspaceTagBackend::index_workspace_tag(
                    self.backend.db_platform.as_ref(),
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

    // check whether the commit exists
    pub fn check_commit<S: AsRef<str>>(
        &self,
        commit_id: S,
    ) -> Result<(), PmrRepoError> {
        let workspace_id = self.workspace.id();
        let repo = self.repo()?;
        get_commit(&repo, workspace_id, Some(commit_id.as_ref()))?;
        Ok(())
    }

    // commit_id/path should be a pathinfo struct?
    pub fn pathinfo<S: Into<String>>(
        &'repo self,
        commit_id: Option<S>,
        path: Option<S>,
    ) -> Result<GitHandleResult<'repo>, PmrRepoError>
    {
        let commit_id = commit_id.map(|s| s.into());
        let path = path.map(|s| s.into());

        let workspace_id = self.workspace.id();
        let repo = self.repo()?;
        let commit = get_commit(&repo, workspace_id, commit_id.as_deref())?;
        match commit {
            None => Ok(GitHandleResult {
                backend: &self.backend,
                repo: self.repo.get()
                    .expect("OnceLock should have been set with the self.repo()")
                    .as_ref()
                    .expect("Valid repo was resolved by here"),
                commit: None,
                target: None,
                workspace: &self.workspace,
            }),
            Some(commit) => {
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
                Ok(GitHandleResult {
                    backend: &self.backend,
                    repo: &self.repo.get()
                        .expect("OnceLock should have been set with the self.repo()")
                        .as_ref()
                        .expect("Valid repo was resolved by here"),
                    commit: Some(commit.into()),
                    target: Some(target),
                    workspace: &self.workspace,
                })
            }
        }
    }

    pub fn loginfo(
        &self,
        commit_id: Option<&str>,
        path: Option<&'repo str>,
        // TODO need to provide a way to skip
        count: Option<usize>,
    ) -> Result<LogInfo, PmrRepoError> {
        let workspace_id = self.workspace.id();
        let repo = self.repo()?;
        let commit = get_commit(&repo, workspace_id, commit_id)?;
        if commit.is_none() {
            return Ok(LogInfo { entries: Vec::new() });
        }
        let commit = commit.expect("None case should have been handled");
        let mut filter = PathFilter::new(&repo, path);
        let log_entry_iter = repo.rev_walk([commit.id])
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
        let repo = self.repo()?;
        let result = get_commit(&repo, workspace_id, commit_id)?
            .map(|commit| files(&commit))
            .unwrap_or_else(|| Ok(Vec::new()));
        result
    }

    pub fn checkout(
        &self,
        commit_id: Option<&str>,
        dest_dir: &Path,
    ) -> Result<(), PmrRepoError> {
        let workspace_id = self.workspace.id();
        let repo = self.repo()?;
        let commit = get_commit(
            &repo,
            workspace_id,
            // force a HEAD commit at the minimum to trigger error handling
            Some(commit_id.unwrap_or("HEAD")),
        )?.expect("a commit was expected with Some(commit_id) provided");
        checkout(&repo, &commit, dest_dir)
    }

}

impl<'repo> GitHandleResult<'repo> {
    pub fn repo(&self) -> Repository {
        self.repo.to_thread_local()
    }

    pub fn commit(&self, repo: &'repo Repository) -> Option<Commit<'repo>> {
        self.commit
            .as_ref()
            .map(|commit| commit.clone()
                .attach(repo)
                .into_commit())
    }

    pub fn path(&self) -> Option<&str> {
        self.target
            .as_ref()
            .map(|target| match target {
                GitResultTarget::Object(object) => object.path.as_str(),
                GitResultTarget::RemoteInfo(remote_info) => remote_info.path.as_str(),
            })
    }

    // TODO could use an TryInto<PathObject<'repo>> or something along that line
    // for getting the final result.
    pub fn target(&self) -> Option<&GitResultTarget> {
        self.target.as_ref()
    }

    pub fn workspace(&'repo self) -> &WorkspaceRef<'repo> {
        &self.workspace
    }

    #[async_recursion]
    pub async fn stream_blob(
        &self,
        mut writer: impl Write + Send + 'async_recursion,
    ) -> Result<usize, PmrRepoError> {
        match &self.target {
            None => Err(ContentError::Invalid {
                workspace_id: self.workspace.id(),
                oid: self.commit.as_ref().map(|c| c.id.to_string()).unwrap_or("<None>".to_string()),
                path: self.path().map(str::to_string).unwrap_or("<None>".to_string()),
                msg: format!("expected to be a blob"),
            }.into()),
            Some(GitResultTarget::Object(object)) => match object.object.kind {
                Kind::Blob => Ok(writer.write(&object.object.data)?),
                _ => Err(ContentError::Invalid {
                    workspace_id: self.workspace.id(),
                    oid: self.commit.as_ref().map(|c| c.id.to_string()).unwrap_or("<None>".to_string()),
                    path: self.path().map(str::to_string).unwrap_or("<None>".to_string()),
                    msg: format!("expected to be a blob"),
                }.into()),
            },
            Some(GitResultTarget::RemoteInfo(RemoteInfo { location, commit, subpath, .. })) => {
                let workspaces = WorkspaceBackend::list_workspace_by_url(
                    self.backend.db_platform.as_ref(), &location,
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
                let git_result = handle.pathinfo(Some(commit), Some(subpath))?;
                git_result.stream_blob(writer).await
            },
        }
    }

    /// Return the list of files associated with the commit that this
    /// `GitHandleResult` is associated with.
    pub fn files(
        &self,
        repo: &'repo Repository,
    ) -> Result<Vec<String>, PmrRepoError> {
        Ok(self.commit(repo)
            .as_ref()
            .map(files)
            .transpose()?
            .unwrap_or_else(|| Vec::new()))
    }
}

