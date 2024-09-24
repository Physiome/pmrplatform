use pmrac::{
    Platform,
    error::{
        AuthenticationError,
        Error,
        PasswordError,
    },
    password::Password,
};

use test_pmr::ac::create_sqlite_platform;

async fn basic_lifecycle(purge: bool) -> anyhow::Result<()> {
    let platform: Platform = create_sqlite_platform(purge).await?;
    let new_user = platform.create_user("admin").await?;
    let admin = platform.get_user(new_user.id()).await?;
    assert_eq!(admin.id(), new_user.id());
    assert_eq!(admin.name(), "admin");

    assert!(matches!(
        platform.verify_user_id_password(
            admin.id(),
            "New",
        ).await,
        Err(Error::Password(e)) if e == PasswordError::NotVerifiable,
    ));

    assert!(matches!(
        admin.reset_password(
            "hunter2",
            "hunter",
        ).await,
        Err(Error::Password(e)) if e == PasswordError::Mismatched,
    ));
    admin.reset_password(
        "hunter2",
        "hunter2",
    ).await?;
    assert!(matches!(
        admin.reset_password(
            "hunter2",
            "hunter2",
        ).await,
        Err(Error::Password(e)) if e == PasswordError::Existing,
    ));

    assert!(platform.verify_user_id_password(admin.id(), "hunter2").await.is_ok());

    assert!(matches!(
        admin.update_password(
            "hunter",
            "not_password",
            "NOT_password",
        ).await,
        Err(Error::Password(e)) if e == PasswordError::Mismatched,
    ));

    assert!(matches!(
        admin.update_password(
            "hunter",
            "password",
            "password",
        ).await,
        Err(Error::Password(e)) if e == PasswordError::Wrong,
    ));

    admin.update_password(
        "hunter2",
        "Password",
        "Password",
    ).await?;

    assert!(platform.verify_user_id_password(admin.id(), "Password").await.is_ok());

    platform.force_user_id_password(admin.id(), Password::Reset).await?;

    assert!(matches!(
        platform.verify_user_id_password(
            admin.id(),
            "Reset",
        ).await,
        Err(Error::Password(e)) if e == PasswordError::NotVerifiable,
    ));

    Ok(())
}

#[async_std::test]
async fn basic_lifecycle_password_autopurge() -> anyhow::Result<()> {
    basic_lifecycle(true).await
}

#[async_std::test]
async fn basic_lifecycle_no_autopurge() -> anyhow::Result<()> {
    basic_lifecycle(false).await
}

async fn error_handling(purge: bool) -> anyhow::Result<()> {
    let platform: Platform = create_sqlite_platform(purge).await?;
    let new_user = platform.create_user("admin").await?;
    let admin = platform.get_user(new_user.id()).await?;

    platform.force_user_id_password(admin.id(), Password::Restricted).await?;
    assert!(matches!(
        platform.new_user_id_password(admin.id(), "Restricted").await,
        Err(Error::Authentication(e)) if e == AuthenticationError::Restricted,
    ));
    platform.force_user_id_password(admin.id(), Password::Misconfigured).await?;
    assert!(matches!(
        platform.new_user_id_password(admin.id(), "Misconfigured").await,
        Err(Error::Misconfiguration),
    ));

    Ok(())
}

#[async_std::test]
async fn error_handling_password_autopurge() -> anyhow::Result<()> {
    error_handling(true).await
}

#[async_std::test]
async fn error_handling_no_autopurge() -> anyhow::Result<()> {
    error_handling(false).await
}
