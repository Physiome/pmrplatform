use gix::{
    Object,
    Repository,
    actor::SignatureRef,
};
use crate::error::GixError;

pub(crate) fn rev_parse_single<'a>(
    repo: &'a Repository,
    commit_id: &'a str,
) -> Result<Object<'a>, GixError> {
    Ok(repo.rev_parse_single(commit_id)?.object()?)
}

pub(crate) fn format_signature_ref(
    value: &SignatureRef,
) -> String {
    format!("{} <{}>", value.name, value.email)
}

pub(crate) struct PathFilter<'a> {
    repo: &'a Repository,
    path: Option<&'a str>,
}

impl<'a> PathFilter<'a> {
    pub(crate) fn new(
        repo: &'a Repository,
        path: Option<&'a str>,
    ) -> Self {
        PathFilter {
            repo: repo,
            path: path,
        }
    }

    pub(crate) fn check(
        &mut self,
        info: &gix::revision::walk::Info,
    ) -> bool {
        self.path
            .map(|path| {
                let oid = self.repo
                    .rev_parse_single(
                        format!("{}:{}", info.id, path).as_str()
                    )
                    .ok();
                // any mismatches will be safe to skip (e.g. when the
                // path does not exist in the commit).
                !info.parent_ids
                    .iter()
                    .all(|id| self.repo
                        .rev_parse_single(
                            format!("{}:{}", id, path).as_str()
                        ).ok() == oid
                    )
            })
            .unwrap_or(true)
    }
}
