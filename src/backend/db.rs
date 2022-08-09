use sqlx::{
    Pool,
    sqlite::SqlitePool
};
use std::sync::Arc;

pub trait PmrBackend {}

pub struct Backend<T> {
    pub pool: Arc<T>,
}

impl<DB: sqlx::Database> Backend<Pool<DB>> {
    pub fn bind(pool: Pool<DB>) -> Self {
        Self {
            pool: Arc::new(pool),
        }
    }

    pub async fn from_url(url: &str) -> anyhow::Result<Self> {
        let pool = Pool::<DB>::connect(url).await?;
        Ok(Self::bind(pool))
    }
}

pub type SqliteBackend = Backend<SqlitePool>;

impl PmrBackend for SqliteBackend {}
