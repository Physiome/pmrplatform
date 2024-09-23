use pmrac::{
    Platform,
    error::{
        Error,
        PasswordError,
    },
};

use test_pmr::ac::create_sqlite_platform;

#[async_std::test]
async fn basic_usage() -> anyhow::Result<()> {
    let platform: Platform = create_sqlite_platform().await?;
    let new_user = platform.create_user("admin").await?;
    let admin = platform.get_user(new_user.id()).await?;
    assert_eq!(admin.id(), new_user.id());
    assert_eq!(admin.name(), "admin");

    platform.set_user_id_password(admin.id(), "hunter2").await?;
    assert!(platform.validate_user_id_password(admin.id(), "hunter2").await?);

    assert!(matches!(
        admin.update_password(
            "hunter",
            "not_password",
            "NOT_password",
        ).await,
        Err(Error::PasswordError(e)) if e == PasswordError::WrongPassword,
    ));

    assert!(matches!(
        admin.update_password(
            "hunter2",
            "password",
            "Password",
        ).await,
        Err(Error::PasswordError(e)) if e == PasswordError::MismatchedPassword,
    ));

    admin.update_password(
        "hunter2",
        "Password",
        "Password",
    ).await?;

    assert!(platform.validate_user_id_password(admin.id(), "Password").await?);

    Ok(())
}
