use async_trait::async_trait;
use futures::future;
use pmrcore::{
    error::BackendError,
    exposure::profile::{
        ExposureFileProfile,
        traits::ExposureFileProfileBackend,
    },
    task_template::UserInputMap,
};

use crate::backend::db::SqliteBackend;

async fn set_ef_profile_sqlite(
    sqlite: &SqliteBackend,
    exposure_file_id: i64,
    profile_id: i64,
) -> Result<(), BackendError> {
    sqlx::query!(
        r#"
INSERT INTO exposure_file_profile (
    exposure_file_id,
    profile_id
)
VALUES (?1, ?2)
ON CONFLICT(exposure_file_id)
DO UPDATE SET
    profile_id = ?2
        "#,
        exposure_file_id,
        profile_id,
    )
    .execute(&*sqlite.pool)
    .await?;
    Ok(())
}

async fn get_ef_profile_core_sqlite(
    sqlite: &SqliteBackend,
    exposure_file_id: i64,
) -> Result<ExposureFileProfile, BackendError> {
    Ok(sqlx::query!(r#"
SELECT
    id,
    exposure_file_id,
    profile_id
FROM exposure_file_profile
WHERE id = ?1
"#,
        exposure_file_id,
    )
        .map(|row| ExposureFileProfile::new(
            row.id,
            row.exposure_file_id,
            row.profile_id,
        ))
        .fetch_one(&*sqlite.pool)
        .await?
    )
}

async fn get_ef_profile_sqlite(
    sqlite: &SqliteBackend,
    exposure_file_id: i64,
) -> Result<ExposureFileProfile, BackendError> {
    let mut rec = get_ef_profile_core_sqlite(
        sqlite,
        exposure_file_id,
    ).await?;

    rec.user_input = sqlx::query!(r#"
SELECT
    arg_id,
    input
FROM exposure_file_profile_input
WHERE
    exposure_file_profile_id = ?1
"#,
        rec.id,
    )
        .map(|row| (
            row.arg_id,
            row.input,
        ))
        .fetch_all(&*sqlite.pool)
        .await?
        .into_iter()
        .collect();

    Ok(rec)
}

async fn update_ef_user_input_sqlite(
    sqlite: &SqliteBackend,
    exposure_file_id: i64,
    user_input: &UserInputMap,
) -> Result<(), BackendError> {
    let rec = get_ef_profile_core_sqlite(
        sqlite,
        exposure_file_id,
    ).await?;

    let exposure_file_profile_id = rec.id;

    future::try_join_all(
        user_input.iter()
            .map(|(arg_id, input)| async move {
                sqlx::query!(
                    r#"
INSERT INTO exposure_file_profile_input (
    exposure_file_profile_id,
    arg_id,
    input
)
VALUES (?1, ?2, ?3)
ON CONFLICT(exposure_file_profile_id, arg_id)
DO UPDATE SET
    arg_id = ?2,
    input = ?3
"#,
                    exposure_file_profile_id,
                    arg_id,
                    input,
                )
                    .execute(&*sqlite.pool)
                    .await?;
                Ok::<(), BackendError>(())
            })
    )
        .await?;
    Ok(())
}

#[async_trait]
impl ExposureFileProfileBackend for SqliteBackend {
    async fn set_ef_profile(
        &self,
        exposure_file_id: i64,
        profile_id: i64,
    ) -> Result<(), BackendError> {
        set_ef_profile_sqlite(
            &self,
            exposure_file_id,
            profile_id,
        ).await
    }

    async fn get_ef_profile(
        &self,
        exposure_file_id: i64,
    ) -> Result<ExposureFileProfile, BackendError> {
        get_ef_profile_sqlite(
            &self,
            exposure_file_id,
        ).await
    }

    async fn update_ef_user_input(
        &self,
        exposure_file_id: i64,
        user_input: &UserInputMap,
    ) -> Result<(), BackendError> {
        update_ef_user_input_sqlite(
            &self,
            exposure_file_id,
            user_input,
        ).await
    }
}

#[cfg(test)]
pub(crate) mod testing {
    use pmrcore::{
        exposure::profile::traits::ExposureFileProfileBackend,
        profile::traits::{
            ProfileBackend,
            ProfileViewsBackend,
            ViewTaskTemplateBackend,
        },
        task_template::UserInputMap,
    };
    use crate::backend::db::{
        Profile,
        SqliteBackend,
    };
    use crate::model::db::sqlite::{
        workspace::testing::make_example_workspace,
        exposure::testing::make_example_exposure,
        exposure_file::testing::make_example_exposure_file,
    };

    #[async_std::test]
    async fn test_basic() -> anyhow::Result<()> {
        let backend = SqliteBackend::from_url("sqlite::memory:")
            .await?
            .run_migration_profile(Profile::Pmrapp)
            .await?;

        let exposure_file_id = make_example_exposure_file(
            &backend,
            make_example_exposure(
                &backend,
                make_example_workspace(&backend).await?,
            ).await?,
            "README.md"
        ).await?;

        let pb: &dyn ProfileBackend = &backend;
        let profile_id = pb.insert_profile("Test Profile", "").await?;
        let empty_profile_id = pb.insert_profile("Empty Profile", "").await?;

        let vttb: &dyn ViewTaskTemplateBackend = &backend;
        let v1 = vttb.insert_view_task_template("view1", "", 1).await?;
        let v2 = vttb.insert_view_task_template("view2", "", 2).await?;

        let pvb: &dyn ProfileViewsBackend = &backend;
        pvb.insert_profile_views(profile_id, v1).await?;
        pvb.insert_profile_views(profile_id, v2).await?;

        let efpb: &dyn ExposureFileProfileBackend = &backend;
        efpb.set_ef_profile(
            exposure_file_id,
            profile_id,
        ).await?;
        let ef_profile = efpb.get_ef_profile(
            exposure_file_id,
        ).await?;

        assert_eq!(ef_profile.exposure_file_id, exposure_file_id);
        assert_eq!(ef_profile.profile_id, profile_id);
        assert_eq!(ef_profile.user_input.len(), 0);

        let user_input = UserInputMap::from([
            (1, "First".to_string()),
            (2, "Alternate".to_string()),
        ]);
        efpb.update_ef_user_input(exposure_file_id, &user_input).await?;

        let ef_profile = efpb.get_ef_profile(
            exposure_file_id,
        ).await?;
        assert_eq!(ef_profile.user_input, user_input);

        efpb.set_ef_profile(
            exposure_file_id,
            empty_profile_id,
        ).await?;
        let empty_ef_profile = efpb.get_ef_profile(
            exposure_file_id,
        ).await?;

        assert_eq!(empty_ef_profile.exposure_file_id, exposure_file_id);
        assert_eq!(empty_ef_profile.profile_id, empty_profile_id);
        // user input remains unchanged
        assert_eq!(empty_ef_profile.user_input.len(), 2);

        // Updating of user input will only replace existing values
        efpb.update_ef_user_input(
            exposure_file_id,
            &UserInputMap::from([
                (2, "Second".to_string()),
                (3, "Third".to_string()),
            ])
        ).await?;
        let final_ef_profile = efpb.get_ef_profile(
            exposure_file_id,
        ).await?;
        assert_eq!(final_ef_profile.user_input, UserInputMap::from([
            (1, "First".to_string()),
            (2, "Second".to_string()),
            (3, "Third".to_string()),
        ]));

        Ok(())
    }
}
