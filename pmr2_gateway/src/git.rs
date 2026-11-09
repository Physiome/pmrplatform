use axum::{
    body::{Body, HttpBody},
    extract::{FromRequestParts, OriginalUri, Path},
    http::{
        header::{self, HeaderValue},
        request::Parts,
        Request, Response,
    },
};
use camino::{Utf8Path, Utf8PathBuf};
use gix::{ObjectDetached, ThreadSafeRepository};
use pmrcore::repo::ArchiveFormat;
use std::{
    convert::Infallible,
    future::Future,
    io::{self, Cursor, Seek, SeekFrom, Write},
    path::PathBuf,
    pin::Pin,
    str::FromStr,
    sync::Arc,
    task::{Context, Poll},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use tokio::io::{AsyncRead, AsyncSeek, ReadBuf};
use tower::Service;
use tower_http::services::{
    fs::{Backend, File, Metadata},
    ServeDir,
};

use crate::{
    gateway::Gateway,
    path::CachePathBuf,
};

// Emulate the browser view identifier defined in Plone.
#[derive(Clone, Copy, Debug)]
enum View {
    Download,
    Rawfile,
    ArchiveTgz,
    ArchiveZip,
}

impl FromStr for View {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_ref() {
            "download" => Ok(View::Download),
            "rawfile" => Ok(View::Rawfile),
            "archive-tgz" => Ok(View::ArchiveTgz),
            "archive-zip" => Ok(View::ArchiveZip),
            _ => Err(())
        }
    }
}

#[derive(Clone, Debug)]
enum ViewParam {
    Download {
        prefix: String,
        workspace_id: String,
        commit_id: String,
        path: String,
    },
    Rawfile {
        prefix: String,
        workspace_id: String,
        commit_id: String,
        path: String,
    },
    ArchiveTgz {
        prefix: String,
        workspace_id: String,
        commit_id: String,
    },
    ArchiveZip {
        prefix: String,
        workspace_id: String,
        commit_id: String,
    },
}

#[derive(Debug)]
pub struct GitGateway<B> {
    gateway: Gateway<B>,
    pmr2_fs_root: Arc<Utf8PathBuf>,
    // This is a clutch to deal with the lack of `InnerPath` in axum-0.8 which will be relased in `axum-0.9`,
    // as the nested version will have an additional parameter.
    nested: bool,
    // This is ultimately a hint to set certain headers or to disambiguate which archive to generate; the
    // resolution is ultimately done by the extracted path implied by how the routes are set up.
    view: Option<View>,
}

impl<B> GitGateway<B> {
    pub fn new(gateway: Gateway<B>, pmr2_fs_root: impl Into<Utf8PathBuf>) -> Self {
        Self {
            gateway,
            pmr2_fs_root: Arc::new(pmr2_fs_root.into()),
            nested: false,
            view: None,
        }
    }

    pub fn nested(&self) -> Self {
        Self {
            gateway: self.gateway.clone(),
            pmr2_fs_root: self.pmr2_fs_root.clone(),
            nested: true,
            view: self.view,
        }
    }

    pub fn for_view(mut self, view: &str) -> Self {
        self.view = View::from_str(view)
            .map_err(|_| {
                #[cfg(feature = "trace")]
                tracing::warn!(
                    "provided view={view} is not supported, requests will pass through to underlying proxy"
                );
            })
            .ok();
        self
    }

    async fn resolve(&self, mut parts: &mut Parts) -> Option<(CachePathBuf, ViewParam)> {
        let request_uri = parts.extensions
            .get::<OriginalUri>()
            .map(|u| u.0.clone())
            .unwrap_or_else(|| parts.uri.clone());
        let prefix = &request_uri.path()[0..request_uri.path().len() - parts.uri.path().len()];
        #[cfg(feature = "trace")]
        tracing::trace!("GitGateway at request_uri={request_uri}, prefix={prefix}");

        if self.nested {
            if let Ok(Path((
                _,
                workspace_id,
                commit_id,
                path,
            ))) = Path::<(String, String, String, String)>::from_request_parts(&mut parts, &()).await {
                #[cfg(feature = "trace")]
                tracing::trace!(
                    "download or rawfile workspace_id={workspace_id}, commit_id={commit_id}, path={path}"
                );
                let cache_path = CachePathBuf::new(format!("{prefix}/{workspace_id}"))?;
                let view_param = match self.view? {
                    View::Download => ViewParam::Download {
                        prefix: prefix.to_string(),
                        workspace_id,
                        commit_id,
                        path,
                    },
                    View::Rawfile => ViewParam::Rawfile {
                        prefix: prefix.to_string(),
                        workspace_id,
                        commit_id,
                        path,
                    },
                    _ => return None,
                };
                Some((cache_path, view_param))
            } else if let Ok(Path((
                _,
                workspace_id,
                commit_id,
            ))) = Path::<(String, String, String)>::from_request_parts(&mut parts, &()).await {
                #[cfg(feature = "trace")]
                tracing::trace!("archive workspace_id={workspace_id}, commit_id={commit_id}");
                let cache_path = CachePathBuf::new(format!("{prefix}/{workspace_id}"))?;
                let view_param = match self.view? {
                    View::ArchiveTgz => ViewParam::ArchiveTgz {
                        prefix: prefix.to_string(),
                        workspace_id,
                        commit_id,
                    },
                    View::ArchiveZip => ViewParam::ArchiveZip {
                        prefix: prefix.to_string(),
                        workspace_id,
                        commit_id,
                    },
                    _ => return None,
                };
                Some((cache_path, view_param))
            } else {
                None
            }
        } else {
            if let Ok(Path((
                workspace_id,
                commit_id,
                path,
            ))) = Path::<(String, String, String)>::from_request_parts(&mut parts, &()).await {
                #[cfg(feature = "trace")]
                tracing::trace!(
                    "download or rawfile workspace_id={workspace_id}, commit_id={commit_id}, path={path}"
                );
                let cache_path = CachePathBuf::new(format!("{prefix}/{workspace_id}"))?;
                let view_param = match self.view? {
                    View::Download => ViewParam::Download {
                        prefix: prefix.to_string(),
                        workspace_id,
                        commit_id,
                        path,
                    },
                    View::Rawfile => ViewParam::Rawfile {
                        prefix: prefix.to_string(),
                        workspace_id,
                        commit_id,
                        path,
                    },
                    _ => return None,
                };
                Some((cache_path, view_param))
            } else if let Ok(Path((
                workspace_id,
                commit_id,
            ))) = Path::<(String, String)>::from_request_parts(&mut parts, &()).await {
                #[cfg(feature = "trace")]
                tracing::trace!("archive workspace_id={workspace_id}, commit_id={commit_id}");
                let cache_path = CachePathBuf::new(format!("{prefix}/{workspace_id}"))?;
                let view_param = match self.view? {
                    View::ArchiveTgz => ViewParam::ArchiveTgz {
                        prefix: prefix.to_string(),
                        workspace_id,
                        commit_id,
                    },
                    View::ArchiveZip => ViewParam::ArchiveZip {
                        prefix: prefix.to_string(),
                        workspace_id,
                        commit_id,
                    },
                    _ => return None,
                };
                Some((cache_path, view_param))
            } else {
                None
            }
        }
    }
}

impl<B> Clone for GitGateway<B> {
    fn clone(&self) -> Self {
        Self {
            gateway: self.gateway.clone(),
            pmr2_fs_root: self.pmr2_fs_root.clone(),
            nested: self.nested,
            view: self.view,
        }
    }
}

impl<ReqBody> Service<Request<ReqBody>> for GitGateway<ReqBody>
where
    ReqBody: Send + HttpBody + Unpin + std::fmt::Debug + 'static,
    <ReqBody as HttpBody>::Data: Send,
    <ReqBody as HttpBody>::Error: std::error::Error + Send + Sync,
{
    type Response = Response<Body>;
    type Error = Infallible;
    type Future = Pin<Box<dyn Future<Output = Result<Response<Body>, Infallible>> + Send + 'static>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.gateway.poll_ready(cx)
    }

    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        let mut this = self.clone();

        Box::pin(async move {
            let (mut parts, body) = req.into_parts();

            let workspace_cache = this.resolve(&mut parts).await;

            let blob = if let Some((workspace_cache, view_param)) = workspace_cache {
                // Currently this is a simplified check - we only check for the `index.html` at the workspace
                // root.  A more comprehensive check would be to check the existence of any cached file and
                // managed that, but for now this is sufficient.
                if this.gateway.is_cached_path(&workspace_cache).await {
                    #[cfg(feature = "trace")]
                    tracing::trace!("found cache workspace_cache={workspace_cache}");
                    #[cfg(feature = "trace")]
                    let start = Instant::now();
                    let response = match &view_param {
                        ViewParam::Download { prefix, workspace_id, commit_id, path } => {
                            Blob::git_file(
                                this.pmr2_fs_root
                                    .join(prefix.trim_start_matches("/"))
                                    .join(workspace_id)
                                    .join(".git"),
                                commit_id,
                                path,
                                true,
                            )
                        }
                        ViewParam::Rawfile { prefix, workspace_id, commit_id, path } => {
                            Blob::git_file(
                                this.pmr2_fs_root
                                    .join(prefix.trim_start_matches("/"))
                                    .join(workspace_id)
                                    .join(".git"),
                                commit_id,
                                path,
                                false,
                            )
                        }
                        ViewParam::ArchiveTgz { prefix, workspace_id, commit_id } => {
                            Blob::archive(
                                this.pmr2_fs_root
                                    .join(prefix.trim_start_matches("/"))
                                    .join(workspace_id)
                                    .join(".git"),
                                commit_id,
                                ArchiveFormat::TarGz,
                            )
                        }
                        ViewParam::ArchiveZip { prefix, workspace_id, commit_id } => {
                            Blob::archive(
                                this.pmr2_fs_root
                                    .join(prefix.trim_start_matches("/"))
                                    .join(workspace_id)
                                    .join(".git"),
                                commit_id,
                                ArchiveFormat::Zip,
                            )
                        }
                    };
                    #[cfg(feature = "trace")]
                    tracing::debug!(
                        "GitGateway generated response for {view_param:?} in {}ms",
                        start.elapsed().as_millis(),
                    );
                    response
                } else {
                    #[cfg(feature = "trace")]
                    tracing::trace!("is not cached workspace_cache={workspace_cache}");
                    None
                }
            } else {
                #[cfg(feature = "trace")]
                tracing::trace!("no workspace_cache?");
                None
            };

            let req = Request::from_parts(parts, body);
            if let Some(backend) = blob.map(BlobBackend::from) {
                // TODO Cache invalidation of relevant endpoints within a publicly available workspace.
                #[cfg(feature = "trace")]
                tracing::trace!("GitGateway serving via ServeDir");
                let filename = backend.blob.metadata.filename.as_ref().map(String::clone);
                let (mut parts, body) = ServeDir::with_backend("/", backend)
                    .call(req)
                    .await?
                    .into_parts();
                if let Some(filename) = filename &&
                    let Ok(header_value) = HeaderValue::from_bytes(
                          format!(r#"attachment; filename="{filename}""#).as_bytes()
                    )
                {
                    parts.headers.insert(header::CONTENT_DISPOSITION, header_value);
                }
                Ok(Response::from_parts(parts, Body::new(body)))
            } else {
                #[cfg(feature = "trace")]
                tracing::trace!("GitGateway falling back to Gateway");
                this.gateway.call(req).await
            }
        })
    }
}

#[derive(Clone, Debug)]
struct BlobBackend {
    blob: Blob,
}

impl Backend for BlobBackend {
    type File = BlobCursor;
    type Metadata = BlobMetadata;
    type OpenFuture = Pin<Box<dyn Future<Output = io::Result<Self::File>> + Send>>;
    type MetadataFuture = Pin<Box<dyn Future<Output = io::Result<Self::Metadata>> + Send>>;

    fn open(&self, _path: PathBuf) -> Self::OpenFuture {
        let cursor = self.blob.clone().into();
        Box::pin(async move {
            Ok(cursor)
        })
    }

    fn metadata(&self, _path: PathBuf) -> Self::MetadataFuture {
        let metadata = self.blob.metadata.clone();
        Box::pin(async move {
            Ok(metadata)
        })
    }
}

impl From<Blob> for BlobBackend {
    fn from(blob: Blob) -> Self {
        Self { blob }
    }
}

#[derive(Clone, Debug)]
struct Blob {
    blob: Arc<[u8]>,
    metadata: BlobMetadata,
}

impl Blob {
    fn git_file(
        repo_dir: Utf8PathBuf,
        commit_id: impl AsRef<str>,
        path: impl AsRef<str>,
        download: bool,
    ) -> Option<Self> {
        let path = path.as_ref();
        let commit_id = commit_id.as_ref();
        let filename = download
            .then(|| path.rsplit("/").next())
            .flatten()
            .map(str::to_string);
        let git_handle = GitHandle::new(&repo_dir)
            .debug_ok(|| format!("cannot open git repo: repo_dir={repo_dir}"))?;
        let commit = git_handle.find_commit(commit_id).ok()?;
        let blob = commit.blob(path)
            .debug_ok(|| format!(
                "error encountered while loading file: repo_dir={repo_dir} commit_id={commit_id} path={path}"
            ))??;
        let len = blob.len() as u64;
        let timestamp = commit.timestamp().ok()?;
        Some(Self {
            blob: blob.into(),
            metadata: BlobMetadata {
                filename,
                timestamp,
                len,
            },
        })
    }

    fn archive(
        repo_dir: Utf8PathBuf,
        commit_id: impl AsRef<str>,
        format: ArchiveFormat,
    ) -> Option<Self> {
        let commit_id = commit_id.as_ref();
        let git_handle = GitHandle::new(&repo_dir)
            .debug_ok(|| format!("cannot open git repo: repo_dir={repo_dir}"))?;
        let commit = git_handle.find_commit(commit_id).ok()?;
        let mut output = Cursor::new(<Vec<u8>>::new());
        // The prefix is based off what is currently done in PMR2.
        let basename = repo_dir.parent()?.file_name()?;
        let prefix = Some(format!(
            "{basename}-{}/",
            &commit.commit_id()[..12],
        ));
        commit.archive(&mut output, format, prefix.as_deref(), [])
            .debug_ok(|| format!(
                "failed to generate archive: repo_dir={repo_dir} commit_id={commit_id} format={format:?}"
            ))?;
        let blob = output.into_inner();
        let len = blob.len() as u64;
        // The filename is assumed to be identical to the workspace identifier (that's how PMR2
        // should have located the git repository for a given workspace on the filesystem).
        let filename = Some(match format {
            ArchiveFormat::TarGz => format!("{basename}.tar.gz"),
            ArchiveFormat::Zip => format!("{basename}.zip"),
        });
        let timestamp = commit.timestamp().ok()?;
        Some(Self {
            blob: blob.into(),
            metadata: BlobMetadata {
                filename,
                timestamp,
                len,
            },
        })
    }
}

#[derive(Debug)]
struct BlobCursor {
    cursor: Cursor<Arc<[u8]>>,
    metadata: BlobMetadata,
}

impl From<Blob> for BlobCursor {
    fn from(Blob { blob, metadata }: Blob) -> Self {
        let cursor = Cursor::new(blob);
        Self { cursor, metadata }
    }
}

impl AsyncRead for BlobCursor {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        AsyncRead::poll_read(Pin::new(&mut self.cursor), cx, buf)
    }
}

impl AsyncSeek for BlobCursor {
    fn start_seek(
        mut self: Pin<&mut Self>,
        position: SeekFrom,
    ) -> io::Result<()> {
        AsyncSeek::start_seek(Pin::new(&mut self.cursor), position)
    }

    fn poll_complete(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<io::Result<u64>> {
        AsyncSeek::poll_complete(Pin::new(&mut self.cursor), cx)
    }
}

impl File for BlobCursor {
    type Metadata = BlobMetadata;
    type MetadataFuture<'a> = Pin<Box<dyn Future<Output = io::Result<Self::Metadata>> + Send + 'a>>;

    fn metadata(&self) -> Self::MetadataFuture<'_> {
        Box::pin(async move { Ok(self.metadata.clone()) })
    }
}

#[derive(Clone, Debug)]
struct BlobMetadata {
    // Determines if the content-disposition header should be sent with this filename.
    filename: Option<String>,
    timestamp: i64,
    len: u64,
}

impl Metadata for BlobMetadata {
    fn is_dir(&self) -> bool {
        false
    }

    fn modified(&self) -> io::Result<SystemTime> {
        if self.timestamp < 0 {
            Ok(UNIX_EPOCH - Duration::from_secs(self.timestamp.unsigned_abs()))
        } else {
            Ok(UNIX_EPOCH + Duration::from_secs(self.timestamp.unsigned_abs()))
        }
    }

    fn len(&self) -> u64 {
        self.len
    }
}

#[derive(Debug, thiserror::Error)]
enum GitError {
    #[error("Gix: {0}")]
    Gix(#[from] gix::Error),
    #[error("GixErrorMessage: {0}")]
    GixErrorMessage(#[from] gix::error::Exn<gix::error::Message>),
    #[error("GixExn: {0}")]
    GixExn(#[from] gix::error::Exn),
}

trait LoggedOk<T> {
    fn debug_ok<F, S>(self, f: F) -> Option<T>
    where
        F: Fn() -> S,
        S: std::fmt::Display;
}

#[cfg(feature = "trace")]
impl<T> LoggedOk<T> for Result<T, GitError> {
    fn debug_ok<F, S>(self, f: F) -> Option<T>
    where
        F: Fn() -> S,
        S: std::fmt::Display,
    {
        self.map_err(|_| tracing::debug!("{}", f())).ok()
    }
}

#[cfg(not(feature = "trace"))]
impl<T> LoggedOk<T> for Result<T, GitError> {
    fn debug_ok<F, S>(self, _f: F) -> Option<T>
    where
        F: Fn() -> S,
        S: std::fmt::Display,
    {
        self.ok()
    }
}

// As much as I wanted to reuse `pmrrepo`, I unfortunately made its design too coupled to the backend early on
// as back in those day I was prototyping and didn't put in the time to figure out a better design.  In any
// case just use the gix APIs directly and figure out how this "decoupled" repository type may be reused later
// back in `pmrrepo`.
#[derive(Clone)]
struct GitHandle {
    repo: ThreadSafeRepository,
    // repo_dir: Utf8PathBuf,
}

impl GitHandle {
    fn new(repo_dir: &Utf8Path) -> Result<Self, GitError> {
        let repo = gix::open::Options::isolated()
            .open_path_as_is(true)
            .open(&repo_dir)?;
        Ok(Self {
            // repo_dir: repo_dir.to_owned(),
            repo,
        })
    }

    fn find_commit(
        &self,
        commit_id: impl Into<String>,
    ) -> Result<GitCommitHandle, GitError> {
        let commit_id = commit_id.into();

        let repo = self.repo.to_thread_local();
        let obj = repo.rev_parse_single(commit_id.as_str())?.object()?;
        Ok(GitCommitHandle {
            git_handle: self.clone(),
            commit: obj.detach(),
        })
    }
}

#[derive(Clone)]
struct GitCommitHandle {
    git_handle: GitHandle,
    commit: ObjectDetached,
}

impl GitCommitHandle {
    fn archive(
        &self,
        out: impl Write + Seek,
        format: ArchiveFormat,
        tree_prefix: Option<&str>,
        extra_entries: impl IntoIterator<Item = (String, Vec<u8>)>,
    ) -> Result<(), GitError> {
        let repo = self.git_handle.repo.to_thread_local();
        let commit = self.commit.clone().attach(&repo).try_into_commit()?;
        let tree = commit.tree()?;
        let (mut stream, _) = repo.worktree_stream(tree.id)?;
        for (path, src) in extra_entries.into_iter() {
            stream.add_entry(gix::worktree::stream::AdditionalEntry {
                id: gix::hash::ObjectId::null(gix::hash::Kind::Sha1),
                mode: gix::object::tree::EntryKind::Blob.into(),
                relative_path: path.into(),
                source: gix::worktree::stream::entry::Source::Memory(src),
            });
        }
        let modification_time = self.timestamp()?;
        let options = gix::worktree::archive::Options {
            format: format.into(),
            modification_time,
            tree_prefix: tree_prefix.map(|s| format!("{s}/").into()),
        };
        Ok(repo.worktree_archive(
            stream,
            out,
            gix::progress::Discard,
            &std::sync::atomic::AtomicBool::default(),
            options,
        )?)
    }

    fn blob(&self, path: impl Into<String>) -> Result<Option<Vec<u8>>, GitError> {
        let path = path.into();
        let repo = self.git_handle.repo.to_thread_local();
        let commit = self.commit.clone().attach(&repo).try_into_commit()?;
        let tree = commit.tree()?;
        if let Some(entry) = tree.lookup_entry(path.split("/"))? {
            Ok(Some(entry.object()?
                .try_into_blob()?
                .take_data()
            ))
        } else {
            Ok(None)
        }
    }

    fn commit_id(&self) -> String {
        self.commit.id.to_hex().to_string()
    }

    fn timestamp(&self) -> Result<i64, GitError> {
        let repo = self.git_handle.repo.to_thread_local();
        Ok(self.commit
            .clone()
            .attach(&repo)
            .try_into_commit()?
            .time()?
            .seconds
        )
    }
}
