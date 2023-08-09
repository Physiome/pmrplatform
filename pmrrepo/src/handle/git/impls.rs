use async_recursion::async_recursion;
use futures::stream::{
    StreamExt as _,
    futures_unordered::FuturesUnordered,
};
use gix::{
    object::Kind,
    objs::{
        CommitRef,
        tree::EntryMode,
    },
    traverse::commit::Sorting,
    traverse::tree::Recorder,
};
use pmrmodel_base::{
    git::{
        LogEntryInfo,
        LogInfo,
        ObjectInfo,
        PathObject,
        RemoteInfo,
    },
    platform::Platform,
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
        ExecutionError,
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

impl<P: Platform + Sync> From<&GitHandleResult<'_, '_, P>> for Option<PathObject> {
    fn from(item: &GitHandleResult<P>) -> Self {
        match &item.target {
            GitResultTarget::Object(object) => match gitresult_to_info(
                item,
                object,
            ) {
                Some(ObjectInfo::FileInfo(file_info)) => Some(PathObject::FileInfo(file_info)),
                Some(ObjectInfo::TreeInfo(tree_info)) => Some(PathObject::TreeInfo(tree_info)), _ => None,
            },
            GitResultTarget::RemoteInfo(RemoteInfo { location, commit, path }) => {
                Some(PathObject::RemoteInfo(RemoteInfo {
                    location: location.to_string(),
                    commit: commit.to_string(),
                    path: path.to_string(),
                }))
            },
        }
    }
}

impl<'a, P: Platform + Sync> TryFrom<Handle<'a, P>> for GitHandle<'a, P> {
    type Error = GixError;

    fn try_from(item: Handle<'a, P>) -> Result<Self, GixError> {
        let repo = gix::open::Options::isolated()
            .open_path_as_is(true)
            .open(&item.repo_dir)?
            .to_thread_local();
        Ok(Self {
            backend: item.backend,
            repo_dir: item.repo_dir,
            workspace: item.workspace,
            repo
        })
    }
}

impl<'a, P: Platform + Sync> GitHandle<'a, P> {
    pub(crate) fn new(
        backend: &'a Backend<P>,
        repo_root: PathBuf,
        workspace: WorkspaceRef<'a, P>,
    ) -> Result<Self, GixError> {
        let repo_dir = repo_root.join(workspace.id().to_string());
        let repo = gix::open::Options::isolated()
            .open_path_as_is(true)
            .open(&repo_dir)?
            .to_thread_local();
        Ok(Self { backend, repo_dir, workspace, repo })
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
    pub fn pathinfo<'b>(
        &'b self,
        commit_id: Option<&'b str>,
        path: Option<&'a str>,
    ) -> Result<GitHandleResult<'a, 'b, P>, PmrRepoError> {
        let workspace_id = self.workspace.id();
        let commit = get_commit(&self.repo, workspace_id, commit_id)?;
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
                                workspace_id: workspace_id,
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
                                workspace_id,
                                curr_path.to_str().unwrap(),
                            )?;
                            target = Some(GitResultTarget::RemoteInfo(RemoteInfo {
                                location: location,
                                commit: entry.id().to_string(),
                                path: comps.as_path().to_str().unwrap().to_string(),
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
                        (path, GitResultTarget::Object(object)),
                    None =>
                        // Only way object is None is have target set.
                        (path, target.expect("to be a RemoteInfo")),
                }
            },
        };
        let item = GitHandleResult {
            backend: &self.backend,
            repo: &self.repo,
            commit: commit,
            path: path,
            target: target,
            workspace: &self.workspace,
        };
        Ok(item)
    }

    pub fn loginfo(
        &self,
        commit_id: Option<&str>,
        path: Option<&'a str>,
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

impl<'a, 'b, P: Platform + Sync> GitHandleResult<'a, 'b, P> {

    #[async_recursion(?Send)]
    pub async fn stream_blob(
        &self,
        mut writer: impl Write + 'async_recursion,
    ) -> Result<usize, PmrRepoError> {
        match &self.target {
            GitResultTarget::Object(object) => match object.kind {
                Kind::Blob => Ok(writer.write(&object.data)?),
                _ => Err(ContentError::Invalid {
                    workspace_id: self.workspace.id(),
                    oid: self.commit.id().to_string(),
                    path: self.path.to_string(),
                    msg: format!("expected to be a blob"),
                }.into())
            },
            GitResultTarget::RemoteInfo(RemoteInfo { location, commit, path }) => {
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
                let git_result = handle.pathinfo(Some(&commit), Some(&path))?;
                git_result.stream_blob(writer).await
            },
        }
    }
}

