use gix::{
    Object,
    objs::CommitRef,
};
pub use gix::object::Kind;

// TODO verify that these are going to be sufficient going forward, as
// these function as newtypes that pmrrepo will use but given the orphan
// rules, all the related impls will need to be provided in this package
// so we are doing the bare minimum here.

#[derive(Debug)]
pub struct PathObject<'a> {
    pub path: String,
    pub object: Object<'a>,
}

#[derive(Debug)]
pub struct IdCommitRef<'a> {
    // TODO consider switching this to the original, or reference the
    // version from the original Commit<'repo>?
    // TODO figure out if this even works - the use case is for a cached
    // copy of decoded data that reference the existing but without the
    // overhead of re-decoding already decoded data.
    pub commit_id: String,
    pub commit: CommitRef<'a>,
}

mod impls;
mod util;
