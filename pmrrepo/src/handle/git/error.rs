pub(crate) enum FetchClone {
    Libgit2(git2::Error),
    Message(String),
}
