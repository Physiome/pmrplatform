use async_trait::async_trait;
use pmrcore::{
    error::BackendError,
    profile::ViewTaskTemplate,
    profile::ViewTaskTemplates,
    profile::traits::ProfileViewsBackend,
};

use crate::{
    backend::db::SqliteBackend,
};

async fn insert_profile_views_sqlite(
    sqlite: &SqliteBackend,
    profile_id: i64,
    view_task_template_id: i64,
) -> Result<i64, BackendError> {
    let id = sqlx::query!(
        r#"
INSERT INTO profile_views (
    profile_id,
    view_task_template_id
)
VALUES ( ?1, ?2 )
        "#,
        profile_id,
        view_task_template_id,
    )
    .execute(&*sqlite.pool)
    .await?
    .last_insert_rowid();
    Ok(id)
}

async fn delete_profile_views_sqlite(
    sqlite: &SqliteBackend,
    profile_id: i64,
    view_task_template_id: i64,
) -> Result<bool, BackendError> {
    let rows_affected = sqlx::query!(
        r#"
DELETE FROM
    profile_views
WHERE profile_id = ?1
    AND view_task_template_id = ?2
        "#,
        profile_id,
        view_task_template_id,
    )
    .execute(&*sqlite.pool)
    .await?
    .rows_affected();
    Ok(rows_affected > 0)
}

async fn get_view_task_templates_for_profile_sqlite(
    sqlite: &SqliteBackend,
    profile_id: i64,
) -> Result<ViewTaskTemplates, BackendError> {
    let result = sqlx::query!(
        r#"
SELECT
    vtt.id,
    vtt.view_key,
    vtt.description,
    vtt.task_template_id,
    vtt.updated_ts
FROM view_task_template AS vtt
JOIN profile_views ON vtt.id == profile_views.view_task_template_id
WHERE profile_views.profile_id = ?1
        "#,
        profile_id,
    )
    .map(|row| ViewTaskTemplate {
        id: row.id,
        view_key: row.view_key,
        description: row.description,
        task_template_id: row.task_template_id,
        updated_ts: row.updated_ts,
        task_template: None,
    })
    .fetch_all(&*sqlite.pool)
    .await?;
    Ok(result.into())
}

#[async_trait]
impl ProfileViewsBackend for SqliteBackend {
    // TODO determine if exposing these low level records are necessary.
    async fn insert_profile_views(
        &self,
        profile_id: i64,
        view_task_template_id: i64,
    ) -> Result<i64, BackendError> {
        insert_profile_views_sqlite(
            &self,
            profile_id,
            view_task_template_id,
        ).await
    }

    async fn delete_profile_views(
        &self,
        profile_id: i64,
        view_task_template_id: i64,
    ) -> Result<bool, BackendError> {
        delete_profile_views_sqlite(
            &self,
            profile_id,
            view_task_template_id,
        ).await
    }

    async fn get_view_task_templates_for_profile(
        &self,
        profile_id: i64,
    ) -> Result<ViewTaskTemplates, BackendError> {
        get_view_task_templates_for_profile_sqlite(
            &self,
            profile_id,
        ).await
    }
}

#[cfg(test)]
mod testing {
    use pmrcore::profile::{
        ViewTaskTemplate,
        traits::{
            ProfileBackend,
            ProfileViewsBackend,
            ViewTaskTemplateBackend,
        },
    };
    use crate::backend::db::{
        MigrationProfile::Pmrapp,
        SqliteBackend,
    };

    #[async_std::test]
    async fn test_basic() -> anyhow::Result<()> {
        let backend = SqliteBackend::from_url("sqlite::memory:")
            .await?
            .run_migration_profile(Pmrapp)
            .await?;
        let pb: &dyn ProfileBackend = &backend;
        let p1 = pb.insert_profile("Profile 1", "").await?;
        let p2 = pb.insert_profile("Profile 2", "").await?;
        let p3 = pb.insert_profile("Profile 3", "").await?;
        let p4 = pb.insert_profile("Profile 4", "").await?;

        let vttb: &dyn ViewTaskTemplateBackend = &backend;
        let v1 = vttb.insert_view_task_template("view1", "", 1).await?;
        let v2 = vttb.insert_view_task_template("view2", "", 2).await?;
        let v3 = vttb.insert_view_task_template("view3", "", 3).await?;
        let v4 = vttb.insert_view_task_template("view4", "", 4).await?;
        let v5 = vttb.insert_view_task_template("view5", "", 5).await?;
        let v6 = vttb.insert_view_task_template("view6", "", 6).await?;

        // omit the ids for now.
        let pvb: &dyn ProfileViewsBackend = &backend;
        // Profile 1
        pvb.insert_profile_views(p1, v1).await?;
        pvb.insert_profile_views(p1, v2).await?;
        pvb.insert_profile_views(p1, v6).await?;
        // Profile 2
        pvb.insert_profile_views(p2, v1).await?;
        pvb.insert_profile_views(p2, v3).await?;
        pvb.insert_profile_views(p2, v4).await?;
        pvb.insert_profile_views(p2, v5).await?;
        pvb.insert_profile_views(p2, v6).await?;
        // Profile 3
        pvb.insert_profile_views(p3, v3).await?;
        pvb.insert_profile_views(p3, v6).await?;

        let p1_vttps = pvb.get_view_task_templates_for_profile(p1).await?;
        let p2_vttps = pvb.get_view_task_templates_for_profile(p2).await?;
        let p3_vttps = pvb.get_view_task_templates_for_profile(p3).await?;
        let p4_vttps = pvb.get_view_task_templates_for_profile(p4).await?;

        assert_eq!(p1_vttps.len(), 3);
        assert_eq!(p2_vttps.len(), 5);
        assert_eq!(p3_vttps.len(), 2);
        assert_eq!(p4_vttps.len(), 0);

        pvb.delete_profile_views(p3, v3).await?;
        let p3_vttps = pvb.get_view_task_templates_for_profile(p3).await?;

        assert_eq!(p3_vttps.as_ref(), [ViewTaskTemplate {
            id: v6,
            view_key: "view6".into(),
            description: "".into(),
            task_template_id: 6,
            updated_ts: 1234567890,
            task_template: None,
        }]);

        Ok(())
    }
}
