use async_trait::async_trait;
use pmrcore::{
    error::BackendError,
    idgen::{
        HexId,
        traits::GenAliasBackend,
    },
};

use crate::backend::db::SqliteBackend;

#[async_trait]
impl GenAliasBackend for SqliteBackend {
    async fn next(&self) -> Result<HexId, BackendError> {
        Ok(sqlx::query!("INSERT INTO gen_alias_seq (id) VALUES (NULL)")
            .execute(&*self.pool)
            .await?
            .last_insert_rowid()
            .into()
        )
    }
}

#[cfg(test)]
pub(crate) mod testing {
    use pmrcore::{
        idgen::traits::GenAliasBackend,
        platform::MCPlatform,
    };
    use crate::backend::db::{
        MigrationProfile,
        SqliteBackend,
    };

    #[async_std::test]
    async fn test_basic() -> anyhow::Result<()> {
        let backend = SqliteBackend::from_url("sqlite::memory:")
            .await?
            .run_migration_profile(MigrationProfile::Pmrapp)
            .await?;

        let id = GenAliasBackend::next(&backend).await?;
        assert_eq!(id.to_string(), "1");

        for _ in 1..11 {
            GenAliasBackend::next(&backend).await?;
        }

        let id = GenAliasBackend::next(&backend).await?;
        assert_eq!(id.to_string(), "c");
        Ok(())
    }
}
