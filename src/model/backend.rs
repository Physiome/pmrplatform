use sqlx::sqlite::SqlitePool;
use std::sync::Arc;

pub trait HasPool {}

pub struct Backend<T> {
    pub pool: Arc<T>,
}

impl<T> Backend<T> {
    pub fn new(pool: T) -> Self {
        Self {
            pool: Arc::new(pool),
        }
    }
}

pub type SqliteBackend = Backend<SqlitePool>;

impl HasPool for SqliteBackend {}
