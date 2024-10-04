use pmrcore::ac::{
    agent::Agent,
    role::Role,
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

use test_pmr::ac::{
    create_sqlite_backend,
    create_sqlite_platform,
};

async fn basic_lifecycle(purge: bool) -> anyhow::Result<()> {
    let platform = create_sqlite_platform(purge).await?;
    let new_user = platform.create_user("admin").await?;
    let admin = platform.get_user(new_user.id()).await?;
    assert_eq!(admin.id(), new_user.id());
    assert_eq!(admin.name(), "admin");

    assert!(matches!(
        platform.verify_user_id_password(admin.id(), "New").await,
        Err(Error::Password(e)) if e == PasswordError::NotVerifiable,
    ));
    assert!(matches!(
        platform.authenticate_user("admin", "New").await,
        Err(Error::Password(e)) if e == PasswordError::NotVerifiable,
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
        Err(Error::Password(e)) if e == PasswordError::NotVerifiable,
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

#[async_std::test]
async fn policy() -> anyhow::Result<()> {
    let platform = create_sqlite_platform(true).await?;
    let user = platform.create_user("admin").await?;
    let state = State::Private;
    let role = Role::Manager;

    platform.grant_res_role_to_agent("/", &user, role).await?;
    platform.revoke_res_role_from_agent("/", &user, role).await?;
    platform.assign_policy_to_wf_state(state, role, "", "GET").await?;
    platform.remove_policy_from_wf_state(state, role, "", "GET").await?;

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
    platform.grant_res_role_to_agent(
        "/*",
        admin,
        Role::Manager,
    ).await?;
    platform.grant_role_to_user(
        &user,
        Role::Reader,
    ).await?;
    platform.grant_res_role_to_agent(
        "/item/1",
        &user,
        Role::Owner,
    ).await?;
    platform.assign_policy_to_wf_state(
        State::Published,
        Role::Reader,
        "",
        "GET",
    ).await?;
    platform.assign_policy_to_wf_state(
        State::Private,
        Role::Owner,
        "edit",
        "POST",
    ).await?;
    platform.assign_policy_to_wf_state(
        State::Private,
        Role::Owner,
        "edit",
        "GET",
    ).await?;
    platform.assign_policy_to_wf_state(
        State::Published,
        Role::Owner,
        "edit",
        "GET",
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
        "res_grants": [
            {"res": "/*", "agent": "admin", "role": "Manager"},
            {"res": "/item/1", "agent": "test_user", "role": "Owner"}
        ],
        "role_permits": [
            {"role": "Owner", "endpoint_group": "edit", "method": "GET"},
            {"role": "Owner", "endpoint_group": "edit", "method": "POST"}
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
            {"res": "/*", "agent": "admin", "role": "Manager"},
            {"res": "/item/1", "agent": "test_user", "role": "Owner"}
        ],
        "role_permits": [
            {"role": "Owner", "endpoint_group": "edit", "method": "GET"},
            {"role": "Reader", "endpoint_group": "", "method": "GET"}
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
    platform.assign_policy_to_wf_state(State::Private, Role::Owner, "edit", "GET").await?;
    platform.assign_policy_to_wf_state(State::Private, Role::Owner, "edit", "POST").await?;
    platform.assign_policy_to_wf_state(State::Pending, Role::Reviewer, "", "GET").await?;
    platform.assign_policy_to_wf_state(State::Pending, Role::Reviewer, "", "POST").await?;
    platform.assign_policy_to_wf_state(State::Pending, Role::Reviewer, "edit", "GET").await?;
    platform.assign_policy_to_wf_state(State::Pending, Role::Reviewer, "edit", "POST").await?;
    platform.assign_policy_to_wf_state(State::Published, Role::Owner, "edit", "GET").await?;
    platform.assign_policy_to_wf_state(State::Published, Role::Reader, "", "GET").await?;

    // there is a welcome page for the site that should be readable by all
    platform.set_wf_state_for_res("/welcome", State::Published).await?;

    let admin = platform.create_user("admin").await?;
    admin.reset_password("admin", "admin").await?;
    platform.grant_res_role_to_agent("/*", admin, Role::Manager).await?;

    let reviewer = platform.create_user("reviewer").await?;
    reviewer.reset_password("reviewer", "reviewer").await?;
    // this enables the reviewer being able to review resources under pending state
    platform.grant_role_to_user(&reviewer, Role::Reviewer).await?;
    platform.grant_role_to_user(&reviewer, Role::Reader).await?;
    platform.grant_res_role_to_agent("/profile/reviewer", reviewer, Role::Owner).await?;
    platform.set_wf_state_for_res("/profile/reviewer", State::Private).await?;

    let user = platform.create_user("user").await?;
    user.reset_password("user", "user").await?;
    platform.grant_role_to_user(&user, Role::Reader).await?;
    platform.grant_res_role_to_agent("/profile/user", user, Role::Owner).await?;
    platform.set_wf_state_for_res("/profile/user", State::Private).await?;

    let admin = platform.authenticate_user("admin", "admin").await?;
    let reviewer = platform.authenticate_user("reviewer", "reviewer").await?;
    let user = platform.authenticate_user("user", "user").await?;

    // since the anonymous_reader isn't enabled for the rbac enforcer...
    assert!(!platform.enforce(Agent::Anonymous, "/welcome", "", "GET").await?);
    assert!(platform.enforce(&admin, "/welcome", "", "GET").await?);
    assert!(platform.enforce(&reviewer, "/welcome", "", "GET").await?);
    assert!(platform.enforce(&user, "/welcome", "", "GET").await?);

    assert!(platform.enforce(&admin, "/profile/user", "", "GET").await?);
    assert!(platform.enforce(&user, "/profile/user", "", "GET").await?);
    assert!(!platform.enforce(&reviewer, "/profile/user", "", "GET").await?);

    // create content owned by user
    platform.grant_res_role_to_agent("/news/post/1", &user, Role::Owner).await?;

    // editable by the user while private
    platform.set_wf_state_for_res("/news/post/1", State::Private).await?;
    assert!(platform.enforce(&admin, "/news/post/1", "edit", "POST").await?);
    assert!(platform.enforce(&user, "/news/post/1", "edit", "POST").await?);
    assert!(!platform.enforce(&reviewer, "/news/post/1", "edit", "POST").await?);

    platform.set_wf_state_for_res("/news/post/1", State::Pending).await?;
    assert!(platform.enforce(&admin, "/news/post/1", "edit", "POST").await?);
    assert!(!platform.enforce(&user, "/news/post/1", "edit", "POST").await?);
    assert!(platform.enforce(&reviewer, "/news/post/1", "edit", "POST").await?);
    assert!(!platform.enforce(&reviewer, "/news/post/1", "grant", "POST").await?);
    assert!(!platform.enforce(&reviewer, "/news/post/2", "edit", "POST").await?);

    // Reviewer role can be granted for one specific resource, to address the use
    // case of requring explicit assignments of items for review to specific reviewer.
    let restricted_reviewer = platform.create_user("restricted_reviewer").await?;
    platform.grant_res_role_to_agent("/news/post/2", &restricted_reviewer, Role::Reviewer).await?;
    assert!(!platform.enforce(&restricted_reviewer, "/news/post/2", "edit", "POST").await?);
    platform.set_wf_state_for_res("/news/post/2", State::Pending).await?;
    assert!(platform.enforce(&restricted_reviewer, "/news/post/2", "edit", "POST").await?);
    assert!(!platform.enforce(&restricted_reviewer, "/news/post/1", "edit", "POST").await?);
    // since they were never granted the general reader role, they won't be able to read
    // the welcome page either...
    assert!(!platform.enforce(&restricted_reviewer, "/welcome", "", "GET").await?);

    let res_grants = platform.get_res_grants("/news/post/1").await?;
    assert!(matches!(&res_grants[0].0, Agent::User(user) if user.name == "user"));
    assert!(matches!(&res_grants[0].1, Role::Owner));

    Ok(())
}
