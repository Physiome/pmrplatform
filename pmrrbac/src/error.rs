#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Builder Error")]
    Builder,
    #[error("Missing required policy")]
    PolicyRequired,
    #[cfg(feature = "casbin")]
    #[error(transparent)]
    Casbin(#[from] casbin::Error),
}

