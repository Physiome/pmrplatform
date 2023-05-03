use sqlx::{
    Pool,
    sqlite::SqlitePool
};
use std::sync::Arc;

pub enum Profile {
    Pmrapp,
    Pmrtqs,
}

pub trait PmrBackend {}

#[derive(Clone)]
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

    // TODO how to disambiguate this between database type
    pub async fn run_migration_profile(
        self,
        profile: Profile
    ) -> anyhow::Result<Self>
    where
        <DB as sqlx::Database>::Connection: sqlx::migrate::Migrate,
    {
        match profile {
            Profile::Pmrapp => {
                sqlx::migrate!("migrations/pmrapp").run(&*self.pool).await?;
            }
            Profile::Pmrtqs => {
                sqlx::migrate!("migrations/pmrtqs").run(&*self.pool).await?;
            }
        }
        Ok(self)
    }
}

pub type SqliteBackend = Backend<SqlitePool>;

impl PmrBackend for SqliteBackend {}
