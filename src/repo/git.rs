use anyhow::bail;
use futures::stream::StreamExt;
use futures::stream::futures_unordered::FuturesUnordered;
use std::io::Write;
use git2::{Repository, Blob, Commit, Object, ObjectType, Tree};
use std::path::{Path, PathBuf};

// TODO HasPool trait name is iffy?
use crate::model::backend::HasPool;
use crate::model::workspace::{
    WorkspaceBackend,
    WorkspaceRecord,
};
use crate::model::workspace_sync::{
    WorkspaceSyncBackend,
    WorkspaceSyncStatus,
    fail_sync,
};
use crate::model::workspace_tag::WorkspaceTagBackend;

pub struct GitPmrAccessor<'a, P: HasPool> {
    backend: &'a P,
    git_root: PathBuf,
    workspace: WorkspaceRecord,
}

pub struct GitResultSet<'git_result_set> {
    pub repo: &'git_result_set Repository,
    pub commit: &'git_result_set Commit<'git_result_set>,
    pub path: &'git_result_set str,
    pub object: Object<'git_result_set>,
}

#[derive(Debug)]
pub struct TreeEntryInfo {
    filemode: String,
    kind: String,
    id: String,
    name: String,
}

// For blob?
#[derive(Debug)]
pub enum ObjectInfo {
    FileInfo {
        size: u64,
        binary: bool,
    },
    TreeInfo {
        filecount: u64,
        entries: Vec<TreeEntryInfo>,
    },
    CommitInfo {
        commit_id: String,
        author: String,
        committer: String,
    },
}

fn blob_to_info(blob: &Blob) -> ObjectInfo {
    ObjectInfo::FileInfo {
        size: blob.size() as u64,
        binary: blob.is_binary(),
    }
}

fn tree_to_info(repo: &Repository, tree: &Tree) -> ObjectInfo {
    ObjectInfo::TreeInfo {
        filecount: tree.len() as u64,
        entries: tree.iter().map(|entry| TreeEntryInfo {
            filemode: format!("{:06o}", entry.filemode()),
            kind: entry.kind().unwrap().str().to_string(),
            id: format!("{}", entry.id()),
            name: entry.name().unwrap().to_string(),
        }).collect(),
    }
}

fn commit_to_info(commit: &Commit) -> ObjectInfo {
    ObjectInfo::CommitInfo {
        commit_id: format!("{}", commit.id()),
        author: format!("{}", commit.author()),
        committer: format!("{}", commit.committer()),
    }
}

fn object_to_info(repo: &Repository, git_object: &Object) -> Option<ObjectInfo> {
    // TODO split off to a formatter version?
    // alternatively, produce some structured data?
    match git_object.kind() {
        Some(ObjectType::Blob) => {
            Some(blob_to_info(git_object.as_blob().unwrap()))
        }
        Some(ObjectType::Tree) => {
            Some(tree_to_info(&repo, git_object.as_tree().unwrap()))
        }
        Some(ObjectType::Commit) => {
            Some(commit_to_info(git_object.as_commit().unwrap()))
        }
        Some(ObjectType::Tag) => {
            None
        }
        Some(ObjectType::Any) | None => {
            None
        }
    }
}

pub fn stream_git_result_set(mut writer: impl Write, git_result_set: &GitResultSet) -> () {
    // TODO split off to a formatter version?
    // alternatively, produce some structured data?
    writer.write(format!("
        have repo at {:?}
        have commit {:?}
        have commit_object {:?}
        using repopath {:?}
        have git_object {:?}
        \n",
        git_result_set.repo.path(),
        &git_result_set.commit.id(),
        commit_to_info(&git_result_set.commit),
        git_result_set.path,
        object_to_info(&git_result_set.repo, &git_result_set.object),
    ).as_bytes()).unwrap();
}

pub fn stream_blob(mut writer: impl Write, blob: &Blob) -> std::result::Result<usize, std::io::Error> {
    writer.write(blob.content())
}

pub fn stream_git_result_set_blob(writer: impl Write, git_result_set: &GitResultSet) -> anyhow::Result<()> {
    match git_result_set.object.kind() {
        Some(ObjectType::Blob) => {
            match git_result_set.object.as_blob() {
                Some(blob) => {
                    stream_blob(writer, blob)?;
                    Ok(())
                }
                None => bail!("failed to get blob from object")
            }
        }
        Some(_) | None => {
            bail!("target is not a git blob")
        }
    }
}

impl<'a, P: HasPool + WorkspaceBackend + WorkspaceSyncBackend + WorkspaceTagBackend> GitPmrAccessor<'a, P> {
    pub fn new(backend: &'a P, git_root: PathBuf, workspace: WorkspaceRecord) -> Self {
        Self {
            backend: &backend,
            git_root: git_root,
            workspace: workspace,
        }
    }

    pub async fn git_sync_workspace(&self) -> anyhow::Result<()> {
        let repo_dir = self.git_root.join(self.workspace.id.to_string());
        let repo_check = Repository::open_bare(&repo_dir);

        info!("Syncing local {:?} with remote <{}>...", repo_dir, &self.workspace.url);
        let sync_id = WorkspaceSyncBackend::begin_sync(self.backend, self.workspace.id).await?;
        match repo_check {
            Ok(repo) => {
                info!("Found existing repo at {:?}, synchronizing...", repo_dir);
                let mut remote = repo.find_remote("origin")?;
                match remote.fetch(&[] as &[&str], None, None) {
                    Ok(_) => info!("Repository synchronized"),
                    Err(e) => fail_sync(self.backend, sync_id, format!("Failed to synchronize: {}", e)).await?,
                };
            },
            Err(ref e) if e.class() == git2::ErrorClass::Repository => fail_sync(
                self.backend, sync_id, format!(
                    "Invalid data at local {:?} - expected bare repo", repo_dir)).await?,
            Err(_) => {
                info!("Cloning new repository at {:?}...", repo_dir);
                let mut builder = git2::build::RepoBuilder::new();
                builder.bare(true);
                match builder.clone(&self.workspace.url, &repo_dir) {
                    Ok(_) => info!("Repository cloned"),
                    Err(e) => fail_sync(self.backend, sync_id, format!("Failed to clone: {}", e)).await?,
                };
            }
        }

        WorkspaceSyncBackend::complete_sync(self.backend, sync_id, WorkspaceSyncStatus::Completed).await?;
        self.index_tags().await?;

        Ok(())
    }

    pub async fn index_tags(&self) -> anyhow::Result<()> {
        let backend = self.backend;
        let git_root = &self.git_root;
        let workspace = &self.workspace;
        let repo_dir = git_root.join(workspace.id.to_string());
        let repo = Repository::open_bare(repo_dir)?;

        // collect all the tags for processing later
        let mut tags = Vec::new();
        repo.tag_foreach(|oid, name| {
            // swapped position for next part.
            tags.push((String::from_utf8(name.into()).unwrap(), format!("{}", oid)));
            true
        })?;

        tags.iter().map(|(name, oid)| async move {
            match WorkspaceTagBackend::index_workspace_tag(backend, workspace.id, &name, &oid).await {
                Ok(_) => info!("indexed tag: {}", name),
                Err(e) => warn!("tagging error: {:?}", e),
            }
        }).collect::<FuturesUnordered<_>>().collect::<Vec<_>>().await;

        Ok(())
    }

    pub async fn get_obj_by_spec(&self, spec: &str) -> anyhow::Result<()> {
        let git_root = &self.git_root;
        let workspace = &self.workspace;
        let repo_dir = git_root.join(workspace.id.to_string());
        let repo = Repository::open_bare(repo_dir)?;
        let obj = repo.revparse_single(spec)?;
        info!("Found object {} {}", obj.kind().unwrap().str(), obj.id());
        info!("{:?}", object_to_info(&repo, &obj));
        Ok(())
    }

    // commit_id/path should be a pathinfo struct?
    pub async fn process_pathinfo<T>(
        &self,
        commit_id: Option<&str>,
        path: Option<&str>,
        processor: fn(&GitResultSet) -> T
    ) -> anyhow::Result<T> {
        let git_root = &self.git_root;
        let workspace = &self.workspace;
        let repo_dir = git_root.join(workspace.id.to_string());
        let repo = Repository::open_bare(repo_dir)?;
        // TODO the default value should be the default (main?) branch.
        // TODO the sync procedure should fast forward of sort
        // TODO the model should have a field for main branch
        let obj = repo.revparse_single(commit_id.unwrap_or("origin/HEAD"))?;
        // TODO streamline this a bit.
        match obj.kind() {
            Some(ObjectType::Commit) => {
                info!("Found {} {}", obj.kind().unwrap().str(), obj.id());
            }
            Some(_) | None => bail!("'{}' does not refer to a valid commit", commit_id.unwrap_or(""))
        }
        let commit = obj.as_commit().unwrap();
        let tree = commit.tree()?;
        info!("Found tree {}", tree.id());
        // TODO only further navigate into tree_entry if path
        let git_object = match path {
            Some(s) => {
                let tree_entry = tree.get_path(Path::new(s))?;
                info!("Found tree_entry {} {}", tree_entry.kind().unwrap().str(), tree_entry.id());
                tree_entry.to_object(&repo)?
            },
            None => {
                info!("No path provided; using root tree entry");
                tree.into_object()
            }
        };
        info!("using git_object {} {}", git_object.kind().unwrap().str(), git_object.id());
        let git_result_set = GitResultSet {
            repo: &repo,
            commit: commit,
            path: path.unwrap_or(""),
            object: git_object,
        };
        Ok(processor(&git_result_set))
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    use async_trait::async_trait;
    use mockall::mock;
    use mockall::predicate::*;
    use tempfile::TempDir;

    // use crate::model::backend::MockHasPool;
    use crate::model::backend::HasPool;
    use crate::model::workspace_tag::WorkspaceTagRecord;
    use crate::model::workspace_sync::WorkspaceSyncRecord;

    mock! {
        Backend {}
        impl HasPool for Backend {}

        #[async_trait]
        impl WorkspaceTagBackend for Backend {
            async fn index_workspace_tag(&self, workspace_id: i64, name: &str, commit_id: &str) -> anyhow::Result<i64>;
            async fn get_workspace_tags(&self, workspace_id: i64) -> anyhow::Result<Vec<WorkspaceTagRecord>>;
        }

        #[async_trait]
        impl WorkspaceBackend for Backend {
            async fn add_workspace(
                &self, url: &str, description: &str, long_description: &str
            ) -> anyhow::Result<i64>;
            async fn update_workspace(
                &self, id: i64, description: &str, long_description: &str
            ) -> anyhow::Result<bool>;
            async fn list_workspaces(&self) -> anyhow::Result<Vec<WorkspaceRecord>>;
            async fn get_workspace_by_id(&self, id: i64) -> anyhow::Result<WorkspaceRecord>;
        }

        #[async_trait]
        impl WorkspaceSyncBackend for Backend {
            async fn begin_sync(&self, workspace_id: i64) -> anyhow::Result<i64>;
            async fn complete_sync(&self, id: i64, status: WorkspaceSyncStatus) -> anyhow::Result<bool>;
            async fn get_workspaces_sync_records(&self, workspace_id: i64) -> anyhow::Result<Vec<WorkspaceSyncRecord>>;
        }
    }

    #[async_std::test]
    async fn test_git_sync_workspace_empty() {
        let (td, _) = crate::test::repo_init(None);
        let mut mock_backend = MockBackend::new();
        mock_backend.expect_begin_sync()
            .times(1)
            .with(eq(0))
            .returning(|_| Ok(1));
        mock_backend.expect_complete_sync()
            .times(1)
            .with(eq(1), eq(WorkspaceSyncStatus::Completed))
            .returning(|_, _| Ok(true));

        let git_root_dir = TempDir::new().unwrap();
        let git_root = git_root_dir.into_path();
        let workspace = WorkspaceRecord {
            id: 0, url: td.path().to_str().unwrap().to_string(), description: None };
        let git_pmr_accessor = GitPmrAccessor::new(&mock_backend, git_root, workspace);
        git_pmr_accessor.git_sync_workspace().await.unwrap();
    }

    #[async_std::test]
    async fn test_git_sync_workspace_with_tag() {
        let (td, repo) = crate::test::repo_init(None);
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

        let git_root_dir = TempDir::new().unwrap();
        let git_root = git_root_dir.into_path();
        let workspace = WorkspaceRecord {
            id: 123, url: td.path().to_str().unwrap().to_string(), description: None };
        let git_pmr_accessor = GitPmrAccessor::new(&mock_backend, git_root, workspace);
        git_pmr_accessor.git_sync_workspace().await.unwrap();
    }

    #[async_std::test]
    async fn test_git_sync_failure_bad_source() {
        let td = TempDir::new().unwrap();
        let err_msg = format!(
            "Failed to clone: could not find repository from '{}'; \
            class=Repository (6); code=NotFound (-3)", td.path().to_str().unwrap());
        let mut mock_backend = MockBackend::new();
        mock_backend.expect_begin_sync()
            .times(1)
            .with(eq(2))
            .returning(|_| Ok(3));
        mock_backend.expect_complete_sync()
            .times(1)
            .with(eq(3), eq(WorkspaceSyncStatus::Error))
            .returning(|_, _| Ok(true));

        let git_root = TempDir::new().unwrap().into_path();
        let workspace = WorkspaceRecord {
            id: 2, url: td.path().to_str().unwrap().to_string(), description: None };
        let git_pmr_accessor = GitPmrAccessor::new(&mock_backend, git_root, workspace);
        assert_eq!(git_pmr_accessor.git_sync_workspace().await.unwrap_err().to_string(), err_msg);
    }

    #[async_std::test]
    async fn test_git_sync_failure_dropped_source() {
        let (td, _) = crate::test::repo_init(None);
        let mut mock_backend = MockBackend::new();
        mock_backend.expect_begin_sync()
            .times(1)
            .with(eq(42))
            .returning(|_| Ok(1));
        mock_backend.expect_complete_sync()
            .times(1)
            .with(eq(1), eq(WorkspaceSyncStatus::Completed))
            .returning(|_, _| Ok(true));

        // helper to deal with moves of the workspace record.
        async fn scoped_sync(
            mock_backend: &MockBackend, url: &str, git_root: &PathBuf
        ) -> anyhow::Result<()> {
            let workspace = WorkspaceRecord {
                id: 42, url: url.to_string(), description: None };
            let git_pmr_accessor = GitPmrAccessor::new(mock_backend, git_root.to_path_buf(), workspace);
            git_pmr_accessor.git_sync_workspace().await
        }

        let git_root_dir = TempDir::new().unwrap();
        let git_root = git_root_dir.into_path().to_owned();

        let td_path = td.path().to_owned();
        let url = td_path.to_str().unwrap();
        assert!(scoped_sync(&mock_backend, url, &git_root).await.is_ok());

        td.close().unwrap();
        mock_backend.checkpoint();
        mock_backend.expect_begin_sync()
            .times(1)
            .with(eq(42))
            .returning(|_| Ok(2));
        mock_backend.expect_complete_sync()
            .times(1)
            .with(eq(2), eq(WorkspaceSyncStatus::Error))
            .returning(|_, _| Ok(true));

        let failed_sync = scoped_sync(&mock_backend, url, &git_root).await;
        assert!(failed_sync.is_err());
        let err_msg = "Failed to synchronize: unsupported URL protocol; class=Net (12)";
        assert_eq!(failed_sync.unwrap_err().to_string(), err_msg);
    }

}
