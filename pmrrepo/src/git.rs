pub use gix::object::Kind;

mod impls;
mod util;

pub use impls::{
    HandleW,
    HandleWR,
    GitResult,
    GitResultTarget,
    WorkspaceGitResult,
    stream_git_result_default,
    stream_git_result_as_json,
    stream_blob,
};
