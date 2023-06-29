use gix::{
    Object,
    Repository,
    actor::SignatureRef,
};
use crate::error::GixError;

pub(super) fn rev_parse_single<'a>(
    repo: &'a Repository,
    commit_id: &'a str,
) -> Result<Object<'a>, GixError> {
    Ok(repo.rev_parse_single(commit_id)?.object()?)
}

pub(super) fn format_signature_ref(
    value: &SignatureRef,
) -> String {
    format!("{} <{}>", value.name, value.email)
}
