use sqlx::SqlitePool;
use std::sync::Arc;

pub struct SqliteBackend {
    pub(crate) pool: Arc<SqlitePool>,
    pub(crate) url: String,
}

mod impls;

pub(crate) mod chrono {
    #[cfg(not(test))]
    pub use ::chrono::Utc;
    #[cfg(test)]
    pub use test_pmr::chrono::Utc;
}
