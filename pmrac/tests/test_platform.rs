use pmrcore::ac::{
    agent::Agent,
    role::Role,
    session::SessionFactory,
    workflow::State,
};
use pmrac::{
    error::{
        AuthenticationError,
        Error,
        PasswordError,
    },
    password::{
        Password,
        PasswordStatus,
    },
    platform::Builder,
};
use pmrrbac::Builder as PmrRbacBuilder;

use test_pmr::{
    ac::{
        create_sqlite_backend,
        create_sqlite_platform,
    },
    chrono::Utc,
    is_send_sync,
};

async fn basic_lifecycle(purge: bool) -> anyhow::Result<()> {
    let platform = create_sqlite_platform(purge).await?;

    assert!(matches!(
        platform.authenticate_user("admin", "admin").await,
        Err(Error::Authentication(AuthenticationError::UnknownUser))
    ));

    let new_user = platform.create_user("admin").await?;
    let admin = platform.get_user(new_user.id()).await?
        .expect("admin wasn't created somehow");
    assert_eq!(admin.id(), new_user.id());
    assert_eq!(admin.name(), "admin");

    assert!(matches!(
        platform.verify_user_id_password(admin.id(), "New").await,
        Err(Error::Authentication(AuthenticationError::Password(e)))
            if e == PasswordError::NotVerifiable,
    ));
    assert!(matches!(
        platform.authenticate_user("admin", "New").await,
        Err(Error::Authentication(AuthenticationError::Password(e)))
            if e == PasswordError::NotVerifiable,
    ));

    let (_, password) = platform.login_status("admin").await?;
    assert!(matches!(password, PasswordStatus::New));

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
    let (_, password) = platform.login_status("admin").await?;
    assert!(matches!(password, PasswordStatus::Hash));

    assert!(platform.authenticate_user("admin", "hunter2").await.is_ok());
    assert!(matches!(
        platform.authenticate_user("admin", "hunter").await,
        Err(Error::Authentication(AuthenticationError::Password(e)))
            if e == PasswordError::Wrong,
    ));

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
        Err(Error::Authentication(AuthenticationError::Password(e)))
            if e == PasswordError::Wrong,
    ));

    admin.update_password(
        "hunter2",
        "Password",
        "Password",
    ).await?;

    assert!(platform.verify_user_id_password(admin.id(), "Password").await.is_ok());
    assert!(platform.authenticate_user("admin", "Password").await.is_ok());

    platform.force_user_id_password(admin.id(), Password::Reset).await?;
    assert!(!platform.authenticate_user("admin", "Password").await.is_ok());

    let (_, password) = platform.login_status("admin").await?;
    assert!(matches!(password, PasswordStatus::Reset));

    assert!(matches!(
        platform.verify_user_id_password(
            admin.id(),
            "Reset",
        ).await,
        Err(Error::Authentication(AuthenticationError::Password(e)))
            if e == PasswordError::NotVerifiable,
    ));

    platform.force_user_id_password(admin.id(), Password::new("resetted")).await?;
    assert!(platform.verify_user_id_password(admin.id(), "resetted").await.is_ok());

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
    let platform = create_sqlite_platform(purge).await?;
    let new_user = platform.create_user("admin").await?;
    let admin = platform.get_user(new_user.id()).await?
        .expect("admin wasn't created somehow");

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

#[async_std::test]
async fn policy() -> anyhow::Result<()> {
    let platform = create_sqlite_platform(true).await?;
    let user = platform.create_user("admin").await?;
    let state = State::Private;
    let role = Role::Manager;

    platform.res_grant_role_to_agent("/", &user, role).await?;
    platform.res_revoke_role_from_agent("/", &user, role).await?;
    platform.assign_policy_to_wf_state(state, role, "").await?;
    platform.remove_policy_from_wf_state(state, role, "").await?;

    Ok(())
}

#[async_std::test]
async fn resource_wf_state() -> anyhow::Result<()> {
    let platform = create_sqlite_platform(true).await?;
    let admin = platform.create_user("admin").await?;
    let user = platform.create_user("test_user").await?;

    platform.grant_role_to_user(
        &admin,
        Role::Reader,
    ).await?;
    platform.res_grant_role_to_agent(
        "/*",
        admin,
        Role::Manager,
    ).await?;
    platform.grant_role_to_user(
        &user,
        Role::Reader,
    ).await?;
    platform.res_grant_role_to_agent(
        "/item/1",
        &user,
        Role::Owner,
    ).await?;
    platform.assign_policy_to_wf_state(
        State::Published,
        Role::Reader,
        "",
    ).await?;
    platform.assign_policy_to_wf_state(
        State::Private,
        Role::Owner,
        "editor_edit",
    ).await?;
    platform.assign_policy_to_wf_state(
        State::Private,
        Role::Owner,
        "editor_view",
    ).await?;
    platform.assign_policy_to_wf_state(
        State::Published,
        Role::Owner,
        "editor_view",
    ).await?;

    assert_eq!(State::Unknown, platform.get_wf_state_for_res("/item/1",).await?);
    platform.set_wf_state_for_res(
        "/item/1",
        State::Private,
    ).await?;
    assert_eq!(State::Private, platform.get_wf_state_for_res("/item/1",).await?);

    let mut policy = platform.generate_policy_for_agent_res(&Agent::Anonymous, "/item/1".into()).await?;
    policy.res_grants.sort_unstable();
    policy.role_permits.sort_unstable();
    assert_eq!(policy, serde_json::from_str(r#"{
        "resource": "/item/1",
        "user_roles": [
        ],
        "res_grants": [],
        "role_permits": [
            {"role": "Owner", "action": "editor_edit"},
            {"role": "Owner", "action": "editor_view"}
        ]
    }"#)?);

    platform.set_wf_state_for_res(
        "/item/1",
        State::Published,
    ).await?;
    let mut policy = platform.generate_policy_for_agent_res(&user.into(), "/item/1".into()).await?;
    policy.res_grants.sort_unstable();
    policy.role_permits.sort_unstable();
    assert_eq!(policy, serde_json::from_str(r#"{
        "resource": "/item/1",
        "user_roles": [
            {"user": "test_user", "role": "Reader"}
        ],
        "res_grants": [
            {"res": "/item/1", "agent": "test_user", "role": "Owner"}
        ],
        "role_permits": [
            {"role": "Owner", "action": "editor_view"},
            {"role": "Reader", "action": ""}
        ]
    }"#)?);

    Ok(())
}

#[async_std::test]
async fn policy_enforcement() -> anyhow::Result<()> {
    let platform = Builder::new()
        .ac_platform(create_sqlite_backend().await?)
        .pmrrbac_builder(PmrRbacBuilder::new_limited())
        .build();
    platform.assign_policy_to_wf_state(State::Private, Role::Owner, "editor_view").await?;
    platform.assign_policy_to_wf_state(State::Private, Role::Owner, "editor_edit").await?;
    platform.assign_policy_to_wf_state(State::Pending, Role::Reviewer, "").await?;
    platform.assign_policy_to_wf_state(State::Pending, Role::Reviewer, "editor_view").await?;
    platform.assign_policy_to_wf_state(State::Pending, Role::Reviewer, "editor_edit").await?;
    platform.assign_policy_to_wf_state(State::Published, Role::Owner, "editor_view").await?;
    platform.assign_policy_to_wf_state(State::Published, Role::Reader, "").await?;

    // there is a welcome page for the site that should be readable by all
    platform.set_wf_state_for_res("/welcome", State::Published).await?;

    let admin = platform.create_user("admin").await?;
    admin.reset_password("admin", "admin").await?;
    platform.res_grant_role_to_agent("/*", admin, Role::Manager).await?;

    let reviewer = platform.create_user("reviewer").await?;
    reviewer.reset_password("reviewer", "reviewer").await?;
    // this enables the reviewer being able to review resources under pending state
    platform.grant_role_to_user(&reviewer, Role::Reviewer).await?;
    platform.grant_role_to_user(&reviewer, Role::Reader).await?;
    platform.res_grant_role_to_agent("/profile/reviewer", reviewer, Role::Owner).await?;
    platform.set_wf_state_for_res("/profile/reviewer", State::Private).await?;

    let user = platform.create_user("user").await?;
    user.reset_password("user", "user").await?;
    platform.grant_role_to_user(&user, Role::Reader).await?;
    platform.res_grant_role_to_agent("/profile/user", user, Role::Owner).await?;
    platform.set_wf_state_for_res("/profile/user", State::Private).await?;

    let admin = platform.authenticate_user("admin", "admin").await?;
    let reviewer = platform.authenticate_user("reviewer", "reviewer").await?;
    let user = platform.authenticate_user("user", "user").await?;

    // since the anonymous_reader isn't enabled for the rbac enforcer...
    assert!(!platform.enforce(Agent::Anonymous, "/welcome", "").await?);
    assert!(platform.enforce(&admin, "/welcome", "").await?);
    assert!(platform.enforce(&reviewer, "/welcome", "").await?);
    assert!(platform.enforce(&user, "/welcome", "").await?);

    assert!(platform.enforce(&admin, "/profile/user", "").await?);
    assert!(platform.enforce(&user, "/profile/user", "").await?);
    assert!(!platform.enforce(&reviewer, "/profile/user", "").await?);

    // create content owned by user
    platform.res_grant_role_to_agent("/news/post/1", &user, Role::Owner).await?;
    platform.res_grant_role_to_agent("/news/post/2", &user, Role::Owner).await?;

    // editable by the user while private
    platform.set_wf_state_for_res("/news/post/1", State::Private).await?;
    assert!(platform.enforce(&admin, "/news/post/1", "editor_edit").await?);
    assert!(platform.enforce(&user, "/news/post/1", "editor_edit").await?);
    assert!(!platform.enforce(&reviewer, "/news/post/1", "editor_edit").await?);

    platform.set_wf_state_for_res("/news/post/1", State::Pending).await?;
    assert!(platform.enforce(&admin, "/news/post/1", "editor_edit").await?);
    assert!(!platform.enforce(&user, "/news/post/1", "editor_edit").await?);
    assert!(platform.enforce(&reviewer, "/news/post/1", "editor_edit").await?);
    assert!(!platform.enforce(&reviewer, "/news/post/1", "grant_edit").await?);
    assert!(!platform.enforce(&reviewer, "/news/post/2", "editor_edit").await?);

    // Reviewer role can be granted for one specific resource, to address the use
    // case of requring explicit assignments of items for review to specific reviewer.
    let restricted_reviewer = platform.create_user("restricted_reviewer").await?;
    platform.res_grant_role_to_agent("/news/post/2", &restricted_reviewer, Role::Reviewer).await?;
    platform.res_grant_role_to_agent("/news/post/3", &restricted_reviewer, Role::Reviewer).await?;
    platform.res_grant_role_to_agent("/news/post/5", &restricted_reviewer, Role::Reviewer).await?;
    assert!(!platform.enforce(&restricted_reviewer, "/news/post/2", "editor_edit").await?);
    platform.set_wf_state_for_res("/news/post/2", State::Pending).await?;
    assert!(platform.enforce(&restricted_reviewer, "/news/post/2", "editor_edit").await?);
    assert!(!platform.enforce(&restricted_reviewer, "/news/post/1", "editor_edit").await?);
    assert!(!platform.enforce(&restricted_reviewer, "/news/post/3", "editor_edit").await?);
    // since they were never granted the general reader role, they won't be able to read
    // the welcome page either...
    assert!(!platform.enforce(&restricted_reviewer, "/welcome", "").await?);

    // retrieve what we have so far
    assert_eq!(
        platform.get_res_grants_for_res("/news/post/1").await?,
        vec![((&user).into(), vec![Role::Owner])],
    );
    let mut res_grants = platform.get_res_grants_for_agent(&(&user).into()).await?;
    res_grants.sort();
    assert_eq!(
        vec![
            ("/news/post/1".to_string(), vec![Role::Owner]),
            ("/news/post/2".to_string(), vec![Role::Owner]),
            ("/profile/user".to_string(), vec![Role::Owner]),
        ],
        res_grants,
    );

    platform.res_grant_role_to_agent("/news/post/2", &restricted_reviewer, Role::Editor).await?;
    let mut res_grants = platform.get_res_grants_for_res("/news/post/2").await?;
    res_grants.sort();
    res_grants.iter_mut().for_each(|(_, roles)| roles.sort());
    assert_eq!(
        vec![
            ((&user).into(), vec![Role::Owner]),
            ((&restricted_reviewer).into(), vec![Role::Editor, Role::Reviewer]),
        ],
        res_grants,
    );

    Ok(())
}

#[async_std::test]
async fn sessions() -> anyhow::Result<()> {
    let platform = Builder::new()
        .ac_platform(create_sqlite_backend().await?)
        .session_factory(
            SessionFactory::new()
                .ts_source(|| Utc::now().timestamp())
        )
        .build();
    let user = platform.create_user("test_user").await?;
    let user_id = user.id();

    // note that a session can be created like this, without a password,
    // typically this lets new users set up their own initial password.
    let session = platform.new_user_session(
        user,
        "localhost".to_string(),
    ).await?;

    // FIXME loading may potentially update last_access_ts later?
    let new_session = platform.load_session(session.session().token).await?;
    assert_eq!(session.user().id(), new_session.user().id());
    assert_eq!(session.session(), new_session.session());

    platform.new_user_session(
        // typically this is done as part of some login workflow
        platform.get_user(user_id).await?
            .expect("user wasn't created somehow"),
        "localhost".to_string(),
    ).await?;
    assert_eq!(2, platform.get_user_sessions(user_id).await?.len());

    // test that saving does bump
    session.save().await?;
    let updated_session = platform.load_session(session.session().token).await?;
    assert_eq!(session.session().origin, updated_session.session().origin);
    assert_ne!(session.session().last_active_ts, updated_session.session().last_active_ts);

    updated_session.logout().await?;

    assert!(platform.load_session(session.session().token).await.is_err());

    Ok(())
}

#[async_std::test]
async fn multiple_sessions() -> anyhow::Result<()> {
    let platform = Builder::new()
        .ac_platform(create_sqlite_backend().await?)
        .session_factory(
            SessionFactory::new()
                .ts_source(|| Utc::now().timestamp())
        )
        .build();
    let user = platform.create_user("test_user").await?;
    let user_id = user.id();

    let s1 = platform.new_user_session(
        platform.get_user(user_id).await?
            .expect("user wasn't created somehow"),
        "site1".to_string()
    ).await?;
    let s2 = platform.new_user_session(
        platform.get_user(user_id).await?
            .expect("user wasn't created somehow"),
        "site2".to_string()
    ).await?;
    let s3 = platform.new_user_session(
        platform.get_user(user_id).await?
            .expect("user wasn't created somehow"),
        "site3".to_string()
    ).await?;

    s2.logout_others().await?;
    assert_eq!(1, platform.get_user_sessions(user_id).await?.len());
    assert!(platform.load_session(s1.session().token).await.is_err());
    assert!(platform.load_session(s2.session().token).await.is_ok());
    assert!(platform.load_session(s3.session().token).await.is_err());

    let s4 = platform.new_user_session(
        platform.get_user(user_id).await?
            .expect("user wasn't created somehow"),
        "site4".to_string()
    ).await?;
    assert_eq!(2, platform.get_user_sessions(user_id).await?.len());

    platform.logout_user(user_id).await?;
    assert_eq!(0, platform.get_user_sessions(user_id).await?.len());
    assert!(platform.load_session(s2.session().token).await.is_err());
    assert!(platform.load_session(s4.session().token).await.is_err());

    Ok(())
}

#[async_std::test]
async fn authenticate_credentials_into_session() -> anyhow::Result<()> {
    let platform = Builder::new()
        .ac_platform(create_sqlite_backend().await?)
        .build();

    let admin = platform.create_user("admin").await?;
    admin.reset_password("admin", "admin").await?;

    assert!(platform
        .authenticate_user_login("admin", "wrong_password", "".to_string())
        .await
        .is_err());

    let session = platform.authenticate_user_login(
        "admin",
        "admin",
        "localhost".to_string(),
    ).await?;

    assert_eq!(session.user().name(), "admin");
    assert_eq!(session.session().origin, "localhost");

    Ok(())
}

#[test]
fn test_send_sync_ctrl() {
    is_send_sync::<pmrac::Platform>();
}
