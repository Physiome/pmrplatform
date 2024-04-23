use async_trait::async_trait;
use pmrcore::{
    error::BackendError,
    profile::Profile,
    profile::traits::ProfileBackend,
};

use crate::{
    backend::db::SqliteBackend,
};

async fn insert_profile_sqlite(
    sqlite: &SqliteBackend,
    title: &str,
    description: &str,
) -> Result<i64, BackendError> {
    let id = sqlx::query!(
        r#"
INSERT INTO profile (
    title,
    description
)
VALUES ( ?1, ?2 )
        "#,
        title,
        description,
    )
    .execute(&*sqlite.pool)
    .await?
    .last_insert_rowid();
    Ok(id)
}

async fn update_profile_by_fields_sqlite(
    sqlite: &SqliteBackend,
    id: i64,
    title: &str,
    description: &str,
) -> Result<bool, BackendError> {
    let rows_affected = sqlx::query!(
        r#"
UPDATE
    profile
SET
    title = ?2,
    description = ?3
WHERE
    id = ?1
        "#,
        id,
        title,
        description,
    )
    .execute(&*sqlite.pool)
    .await?
    .rows_affected();
    Ok(rows_affected > 0)
}

async fn select_profile_by_id_sqlite(
    sqlite: &SqliteBackend,
    id: i64,
) -> Result<Profile, BackendError> {
    let result = sqlx::query!(
        r#"
SELECT
    id,
    title,
    description
FROM profile
WHERE id = ?1
        "#,
        id,
    )
    .map(|row| Profile {
        id: row.id,
        title: row.title,
        description: row.description,
    })
    .fetch_one(&*sqlite.pool)
    .await?;
    Ok(result)
}

#[async_trait]
impl ProfileBackend for SqliteBackend {
    async fn insert_profile(
        &self,
        title: &str,
        description: &str,
    ) -> Result<i64, BackendError> {
        insert_profile_sqlite(&self, title, description).await
    }
    async fn update_profile_by_fields(
        &self,
        id: i64,
        title: &str,
        description: &str,
    ) -> Result<bool, BackendError> {
        update_profile_by_fields_sqlite(&self, id, title, description).await
    }
    async fn select_profile_by_id(
        &self,
        id: i64,
    ) -> Result<Profile, BackendError> {
        select_profile_by_id_sqlite(&self, id).await
    }
}

#[cfg(test)]
mod testing {
    use pmrcore::profile::{
        Profile,
        traits::ProfileBackend,
    };
    use crate::backend::db::{
        Profile::Pmrapp,
        SqliteBackend,
    };

    #[async_std::test]
    async fn test_basic() -> anyhow::Result<()> {
        let backend = SqliteBackend::from_url("sqlite::memory:")
            .await?
            .run_migration_profile(Pmrapp)
            .await?;
        let b: &dyn ProfileBackend = &backend;
        let profile_id = b.insert_profile("Test Profile", "").await?;
        let profile = b.select_profile_by_id(profile_id).await?;
        assert_eq!(profile, Profile {
            id: profile_id,
            title: "Test Profile".to_string(),
            description: "".to_string(),
        });

        assert!(b.update_profile_by_fields(
            profile_id,
            "Updated Title",
            "Updated Description",
        ).await?);
        let profile = b.select_profile_by_id(profile_id).await?;
        assert_eq!(profile, Profile {
            id: profile_id,
            title: "Updated Title".to_string(),
            description: "Updated Description".to_string(),
        });
        Ok(())
    }
}
