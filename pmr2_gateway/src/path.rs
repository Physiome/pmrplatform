use camino::{Utf8Path, Utf8PathBuf};
use percent_encoding::percent_decode;
use tokio::fs::{remove_dir_all, remove_file};
use std::{fmt, ffi::OsStr, path::Path};

/// A resolved cache path.
#[derive(Clone, Default)]
#[repr(transparent)]
pub(crate) struct CachePathBuf(Utf8PathBuf);

impl fmt::Debug for CachePathBuf {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}

impl fmt::Display for CachePathBuf {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl PartialEq for CachePathBuf {
    fn eq(&self, other: &CachePathBuf) -> bool {
        self.0 == other.0
    }
}

impl AsRef<str> for CachePathBuf {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl AsRef<OsStr> for CachePathBuf {
    fn as_ref(&self) -> &OsStr {
        self.0.as_ref()
    }
}

impl AsRef<Path> for CachePathBuf {
    fn as_ref(&self) -> &Path {
        self.0.as_ref()
    }
}

impl AsRef<Utf8Path> for CachePathBuf {
    fn as_ref(&self) -> &Utf8Path {
        self.0.as_ref()
    }
}

impl<'a> PartialEq<&'a str> for CachePathBuf {
    fn eq(&self, other: &&'a str) -> bool {
        <Utf8Path as PartialEq>::eq(&self.0, Utf8Path::new(other))
    }
}

impl CachePathBuf {
    /// Returns the normalized path, with an `index.html` component appended.
    pub(crate) fn new(path: impl AsRef<str>) -> Option<Self> {
        resolve_path(path.as_ref())
    }

    /// Returns the actual path of the resource (i.e. with the `index.html` stripped.
    pub(crate) fn parent(&self) -> Option<Self> {
        Some(Self(self.0
            .parent()?
            .parent()?
            .join("index.html")
            .to_owned()
        ))
    }

    /// Return the path to the header file associated with this cache file.
    pub(crate) fn headers(&self) -> Option<Utf8PathBuf> {
        self.actual().map(|p| p.join("headers"))
    }

    /// Returns the actual path of the resource (i.e. with the `index.html` stripped.
    pub(crate) fn actual(&self) -> Option<&Utf8Path> {
        self.0.parent()
    }

    /// Remove this item from the specified cache.
    pub(crate) async fn invalidate_from(&self, cache_dir: impl AsRef<Utf8Path>) {
        let cache_dir = cache_dir.as_ref();
        let target = cache_dir.join(&self);

        #[cfg_attr(not(feature = "trace"), allow(unused_variables))]
        let result = remove_file(&target).await;
        #[cfg(feature = "trace")]
        match result {
            Ok(_) => tracing::debug!("invalidated cached page {target}"),
            Err(_) => if target.exists() {
                tracing::warn!("failed to invalidate cached page: {target} still exists");
            } else {
                tracing::trace!("cached page {target} previously invalidated");
            },
        }

        if let Some(headers) = self.headers().map(|headers| cache_dir.join(headers)) {
            #[cfg_attr(not(feature = "trace"), allow(unused_variables))]
            let result = remove_file(&headers).await;
            #[cfg(feature = "trace")]
            match result {
                Ok(_) => tracing::debug!("invalidated cached headers {headers}"),
                Err(_) => if target.exists() {
                    tracing::warn!("failed to invalidate cached headers: {headers} still exists");
                } else {
                    tracing::trace!("cached headers {headers} previously invalidated");
                },
            }
        }
    }

    /// Remove this item from the specified cache recursively.
    pub(crate) async fn invalidate_recursive_from(&self, cache_dir: impl AsRef<Utf8Path>) {
        if let Some(actual) = self.actual() {
            let target = cache_dir.as_ref().join(actual);
            #[cfg_attr(not(feature = "trace"), allow(unused_variables))]
            let result = remove_dir_all(&target).await;
            #[cfg(feature = "trace")]
            match result {
                Ok(_) => tracing::debug!("invalidated cache item {target} recursively"),
                Err(_) => if target.exists() {
                    tracing::warn!("failed to invalidate cached item recursively: {target} still exists");
                } else {
                    tracing::trace!("cached item {target} previously invalidated");
                },
            }
        }
    }
}

// This resolves to the filesystem equivalent path.  Given that every resource may have a trailing slash
// and that the resource will have an html presentation, a concrete `index.html` filename will be appended
// to them to match with typical behavior http servers (like `ServeDir`) where it will serve the request
// uri with the trailing slash with that file.  This complicates the issue if a valid uri also contain a
// concrete `index.html` in one of its segment - the resulting segment will have `index.html` as a path
// segment that will be some directory, and that will definitely conflict with the cache file of the same
// name.
//
// The path returned will be some relative path - this is intended as it will be joined with the absolute
// path provided by the service that will consume this value.
//
// Given that no PMR2 resources (currently as of writing) has `index_html` as a directory or a file, this
// function simply converts any segements of `index.html` to `index_html`.  This can conflict with Plone's
// use of this filename but that is restricted to portal content (and view path) which shouldn't cause
// actual issues in the current scenario, but naturally there will be confusion if there was indeed a
// file (or directory) named `index_html` and `index.html` at the same level.
//
// A cleaner workaround would be just to hash all paths into some string, but this way there will be a
// concrete path of the requested resource on the filesystem for direct inspection.  Once this is settled
// using the hash can be done later if the naming conflict (between `index_html` and `index.html`) becomes
// a real issue.
fn resolve_path(path: &str) -> Option<CachePathBuf> {
    let mut segments = Vec::new();
    // This is inline with the behavior of `ServeDir` (as setup with `axum`) - it will resolve `..` that
    // traverse above the root remain within the underlying root and will resolve to the correct file without
    // allowing directory traversal beyond the defined root.
    for segment in percent_decode(path.as_ref())
        .decode_utf8()
        .ok()?
        .split("/")
        .map(|s| {
            if s == "index.html" {
                "index_html".to_owned()
            } else {
                format!("{s}")
            }
        })
    {
        if segment == "." {
            continue;
        } else if segment == ".." {
            segments.pop();
        } else {
            segments.push(segment)
        }
    }
    segments.push("index.html".to_owned());
    Some(CachePathBuf(segments.into_iter().collect::<Utf8PathBuf>()))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn smoke_resolve_path() {
        fn resolve_path_unwrapped(s: &str) -> CachePathBuf {
            resolve_path(s).unwrap()
        }

        assert_eq!(resolve_path_unwrapped("/"), "index.html");
        assert_eq!(resolve_path_unwrapped("/test"), "test/index.html");
        assert_eq!(resolve_path_unwrapped("/test/"), "test/index.html");
        assert_eq!(resolve_path_unwrapped("/test/target"), "test/target/index.html");
        assert_eq!(resolve_path_unwrapped("/test%2ftarget"), "test/target/index.html");
        assert_eq!(resolve_path_unwrapped("/test//target"), "test/target/index.html");
        assert_eq!(resolve_path_unwrapped("/test///target"), "test/target/index.html");
        assert_eq!(resolve_path_unwrapped("/index_html"), "index_html/index.html");
        assert_eq!(resolve_path_unwrapped("/index.html"), "index_html/index.html");
        assert_eq!(resolve_path_unwrapped("/test/index_html"), "test/index_html/index.html");
        assert_eq!(resolve_path_unwrapped("/test/index.html"), "test/index_html/index.html");

        // Segments with special names (i.e. curdir/pardir) should be ignored.
        assert_eq!(resolve_path_unwrapped("/../target"), "target/index.html");
        assert_eq!(resolve_path_unwrapped("/../../../../target"), "target/index.html");
        assert_eq!(resolve_path_unwrapped("/dir/../target"), "target/index.html");
        assert_eq!(
            resolve_path_unwrapped("/workspace/a1/file/../file/main"),
            "workspace/a1/file/main/index.html",
        );
        assert_eq!(
            resolve_path_unwrapped("/workspace/a1/file/../file/main/"),
            "workspace/a1/file/main/index.html",
        );

        // Plone usually has the `/view` suffix being the same as the underlying resource, but for now this
        // resolver ignores this issue as the optimization can be done by the underlying service.
        assert_eq!(resolve_path_unwrapped("/welcome"), "welcome/index.html");
        assert_eq!(resolve_path_unwrapped("/welcome/view"), "welcome/view/index.html");

        // Invalid unicode is decoded to nothing
        assert_eq!(resolve_path("/%e7"), None);
    }

    #[test]
    fn smoke_cache_path_buf() {
        let root = CachePathBuf::new("/").unwrap();
        assert_eq!(root, CachePathBuf("index.html".into()));
        assert_eq!(root.actual(), Some(Utf8Path::new("")));
        assert_eq!(root.parent(), None);

        let path = CachePathBuf::new("/page").unwrap();
        assert_eq!(path, CachePathBuf("page/index.html".into()));
        assert_eq!(path.actual(), Some(Utf8Path::new("page")));
        assert_eq!(path.parent(), Some(CachePathBuf("index.html".into())));

        let nested = CachePathBuf::new("/nested/page").unwrap();
        assert_eq!(nested, CachePathBuf("nested/page/index.html".into()));
        assert_eq!(nested.actual(), Some(Utf8Path::new("nested/page")));
        assert_eq!(nested.parent(), Some(CachePathBuf("nested/index.html".into())));
    }
}
