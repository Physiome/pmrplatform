use pmrcore::platform::PlatformUrl;
use sqlx::{
    Pool,
    sqlite::SqlitePool
};
use std::sync::Arc;

pub enum MigrationProfile {
    Pmrapp,
    Pmrtqs,
}

#[derive(Clone)]
pub struct Backend<T> {
    pub(crate) pool: Arc<T>,
    pub(crate) url: String,
}

impl<T> PlatformUrl for Backend<T> {
    fn url(&self) -> &str {
        self.url.as_ref()
    }
}

impl<DB: sqlx::Database> Backend<Pool<DB>> {
    pub async fn from_url(url: &str) -> Result<Self, sqlx::Error> {
        let pool = Pool::<DB>::connect(url).await?;
        Ok(Self {
            pool: Arc::new(pool),
            url: url.to_string(),
        })
    }

    // TODO how to disambiguate this between database type
    pub async fn run_migration_profile(
        self,
        profile: MigrationProfile
    ) -> Result<Self, sqlx::Error>
    where
        <DB as sqlx::Database>::Connection: sqlx::migrate::Migrate,
    {
        match profile {
            MigrationProfile::Pmrapp => {
                sqlx::migrate!("migrations/pmrapp").run(&*self.pool).await?;
            }
            MigrationProfile::Pmrtqs => {
                sqlx::migrate!("migrations/pmrtqs").run(&*self.pool).await?;
            }
        }
        Ok(self)
    }
}

pub type SqliteBackend = Backend<SqlitePool>;
