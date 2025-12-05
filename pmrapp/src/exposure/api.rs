use leptos::{
    server,
    server_fn,
    server_fn::codec::Rkyv,
};
use pmrcore::{
    alias::AliasEntry,
    exposure::{
        Exposure,
        ExposureFile,
        ExposureFileView,
        profile::ExposureFileProfile,
    },
    profile::{
        Profile,
        UserPromptGroups,
    },
    workspace::Workspace,
};
use serde::{Serialize, Deserialize};
use crate::{
    app::id::Id,
    enforcement::EnforcedOk,
    error::AppError,
};

pub const WIZARD_FIELD_ROUTE: &'static str = "/api/exposure_wizard_field_update";

#[cfg(feature = "ssr")]
mod ssr {
    pub use ammonia::{
        Builder,
        UrlRelative,
    };
    pub use pmrcore::{
        ac::{
            agent::Agent,
            role::Role,
            workflow::State,
        },
        exposure::traits::{
            Exposure as _,
            ExposureBackend,
            ExposureFile as _,
            ExposureFileView as _,
        },
    };
    pub use pmrctrl::error::CtrlError;
    pub use std::borrow::Cow;
    pub use crate::{
        server::{
            log_error,
            platform,
            ac::session,
            exposure::resolve_id,
        },
    };
}
#[cfg(feature = "ssr")]
use self::ssr::*;

pub type Exposures = Vec<AliasEntry<Exposure>>;

#[cfg_attr(feature = "utoipa", utoipa::path(
    post,
    path = "/api/list_exposures",
    responses((
        status = 200,
        description = "List of exposures within an EnforcedOk; wrapped in EnforcedOk due to typical usage as a top level page listing.",
        body = EnforcedOk<Exposures>,
    ), AppError),
    security(
        (),
        ("cookie" = []),
    ),
))]
#[server(endpoint = "list_exposures")]
pub async fn list_exposures() -> Result<EnforcedOk<Exposures>, AppError> {
    let policy_state = session().await?
        .enforcer_and_policy_state("/exposure/", "").await?;
    let platform = platform().await?;
    Ok(policy_state.to_enforced_ok(
        ExposureBackend::list(platform.mc_platform.as_ref())
            .await
            .map_err(|_| AppError::InternalServerError)?
            .into_iter()
            .map(|exposure| AliasEntry {
                alias: exposure.id.to_string(),
                entity: exposure,
            })
            .collect()
    ))
}

#[cfg_attr(feature = "utoipa", utoipa::path(
    post,
    path = "/api/list_aliased_exposures",
    responses((
        status = 200,
        description = "List of exposures with their alias within an EnforcedOk; wrapped in EnforcedOk due to typical usage as a top level page listing.",
        body = EnforcedOk<Exposures>,
    ), AppError),
    security(
        (),
        ("cookie" = []),
    ),
))]
#[server(endpoint = "list_aliased_exposures")]
pub async fn list_aliased_exposures() -> Result<EnforcedOk<Exposures>, AppError> {
    let policy_state = session().await?
        .enforcer_and_policy_state("/exposure/", "").await?;
    let platform = platform().await?;
    let exposures = platform.mc_platform.list_aliased_exposures()
        .await
        .map_err(|_| AppError::InternalServerError)?
        .into_iter()
        .map(|exposure| exposure.map(|entity| entity.into_inner()))
        .collect();
    Ok(policy_state.to_enforced_ok(exposures))
}

#[cfg_attr(feature = "utoipa", utoipa::path(
    post,
    path = "/api/list_aliased_exposures_for_workspace",
    request_body(
        description = r#"
List exposures with their alias for a given workspace.
        "#,
        content((
            Id = "application/json",
            examples(
                ("Example 1" = (
                    summary = "Acquire the list by an alias to an exposure",
                    value = json!({
                        "id": {
                            "Aliased": "beeler_reuter_1977",
                        },
                    })
                )),
            )
        )),
    ),
    responses((
        status = 200,
        description = "List of exposures with their alias for the workspace that was specified.",
        body = Exposures,
    ), AppError),
    security(
        (),
        ("cookie" = []),
    ),
))]
#[server(
    endpoint = "list_aliased_exposures_for_workspace",
    input = server_fn::codec::Json,
)]
pub async fn list_aliased_exposures_for_workspace(
    id: Id,
) -> Result<Exposures, AppError> {
    let workspace_id = crate::server::workspace::resolve_id(id).await?;
    session().await?
        .enforcer(format!("/workspace/{workspace_id}/"), "").await?;
    session().await?
        .enforcer(format!("/exposure/"), "").await?;
    let platform = platform().await?;
    let exposures = platform.mc_platform
        .list_aliased_exposures_for_workspace(workspace_id)
        .await
        .map_err(|_| AppError::InternalServerError)?
        .into_iter()
        .map(|exposure| exposure.map(|entity| entity.into_inner()))
        .collect();
    Ok(exposures)
}

#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Serialize, Deserialize)]
pub struct ExposureInfo {
    pub exposure: Exposure,
    pub exposure_alias: Option<String>,
    pub files: Vec<(String, bool)>,
    pub workspace: Workspace,
    pub workspace_alias: Option<String>,
}

#[cfg_attr(feature = "utoipa", utoipa::path(
    post,
    path = "/api/get_exposure_info",
    request_body(
        description = r#"
Get the top level information of a given exposure.
        "#,
        content((
            Id = "application/json",
            examples(
                ("Example 1" = (
                    summary = "Acquire the exposure information by the alias to the exposure",
                    value = json!({
                        "id": {
                            "Aliased": "c1"
                        },
                    })
                )),
            )
        )),
    ),
    responses((
        status = 200,
        description = "The `ExposureInfo` wrapped by an `EnforcedOk`.",
        body = EnforcedOk<ExposureInfo>,
    ), AppError),
    security(
        (),
        ("cookie" = []),
    ),
))]
#[server(
    endpoint = "get_exposure_info",
    input = server_fn::codec::Json,
)]
pub async fn get_exposure_info(id: Id) -> Result<EnforcedOk<ExposureInfo>, AppError> {
    let id = resolve_id(id).await?;
    let policy_state = session().await?
        .enforcer_and_policy_state(format!("/exposure/{id}/"), "").await?;
    let platform = platform().await?;
    let ctrl = platform.get_exposure(id).await
        .map_err(|_| AppError::InternalServerError)?;
    let files = ctrl.pair_files_info().await
        .map_err(|_| AppError::InternalServerError)?;
    let exposure = ctrl.exposure().clone_inner();
    let workspace = platform.mc_platform
        .get_workspace(exposure.workspace_id)
        .await
        .map_err(|_| AppError::InternalServerError)?
        .into_inner();
    let exposure_alias = ctrl.alias().await
        .map_err(|_| AppError::InternalServerError)?;
    let workspace_alias = platform.mc_platform.get_alias("workspace", exposure.workspace_id)
        .await
        .map_err(|_| AppError::InternalServerError)?;
    Ok(policy_state.to_enforced_ok(ExposureInfo {
        exposure,
        files,
        workspace,
        exposure_alias,
        workspace_alias,
    }))
}

#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub enum ResolvedExposurePath {
    Target(ExposureFile, Result<(ExposureFileView, Option<String>), Vec<String>>),
    Redirect(String),
}

#[cfg_attr(feature = "utoipa", utoipa::path(
    post,
    path = "/api/resolve_exposure_path",
    request_body(
        description = r#"
Attempt to resolve additional information about a path within an exposure.
        "#,
        content((
            Id = "application/json",
            examples(
                ("Example 1" = (
                    summary = "Acquire the exposure information by the alias to the exposure and path",
                    value = json!({
                        "id": {
                            "Aliased": "c1",
                        },
                        "path": "beeler_reuter_1977.cellml",
                    })
                )),
            )
        )),
    ),
    responses((
        status = 200,
        description = "The `ExposureInfo` wrapped by an `EnforcedOk`.",
        body = EnforcedOk<ResolvedExposurePath>,
    ), AppError),
    security(
        (),
        ("cookie" = []),
    ),
))]
#[server(
    endpoint = "resolve_exposure_path",
    input = server_fn::codec::Json,
)]
pub async fn resolve_exposure_path(
    id: i64,
    path: String,
) -> Result<EnforcedOk<ResolvedExposurePath>, AppError> {
    let policy_state = session().await?
        .enforcer_and_policy_state(format!("/exposure/{id}/"), "").await?;
    // TODO when there is a proper error type for id not found, use that
    // TODO ExposureFileView is a placeholder - the real type that should be returned
    // is something that can readily be turned into an IntoView.

    let platform = platform().await?;
    let ec = platform.get_exposure(id).await
        .map_err(|_| AppError::InternalServerError)?;

    match ec.resolve_file_view(path.as_ref()).await {
        (Ok(efc), Ok(efvc)) => {
            // to ensure the views is populated
            efc.exposure_file()
                .views()
                .await
                .map_err(|_| AppError::InternalServerError)?;
            Ok(policy_state.to_enforced_ok(ResolvedExposurePath::Target(
                efc.exposure_file().clone_inner(),
                Ok((
                    efvc.exposure_file_view().clone_inner(),
                    efvc.view_path().map(str::to_string),
                )),
            )))
        },
        (_, Err(CtrlError::None)) => {
            // since the request path has a direct hit on file, doesn't
            // matter if ExposureFileCtrl found or not.
            let exposure = ec.exposure();
            let path = platform.get_workspace(exposure.workspace_id()).await
                .map_err(|_| AppError::InternalServerError)?
                .alias()
                .await
                .map_err(|_| AppError::InternalServerError)?
                .map_or_else(
                    || format!(
                        "/workspace/:/id/{}/rawfile/{}/{path}",
                        exposure.workspace_id(),
                        exposure.commit_id(),
                    ),
                    |alias| format!(
                        "/workspace/{alias}/rawfile/{}/{path}",
                        exposure.commit_id(),
                    ),
                );

            // Not using this built-in axum integration.
            // leptos_axum::redirect(path.as_str());
            //
            // Reason for doing our own thing here is because the target
            // is an endpoint outside of the leptos application and thus
            // not routable/renderable using CSR.
            Ok(policy_state.to_enforced_ok(ResolvedExposurePath::Redirect(path)))
        },
        (Ok(efc), Err(CtrlError::EFVCNotFound(viewstr))) if viewstr == "" => {
            // to ensure the views is populated
            efc.exposure_file()
                .views()
                .await
                .map_err(|_| AppError::InternalServerError)?;
            Ok(policy_state.to_enforced_ok(ResolvedExposurePath::Target(
                efc.exposure_file().clone_inner(),
                Err(efc.exposure_file()
                    .views()
                    .await
                    .map_err(|_| AppError::InternalServerError)?
                    .iter()
                    .filter_map(|v| v.view_key().map(str::to_string))
                    .collect::<Vec<_>>()
                ),
            )))
        },
        // CtrlError::UnknownPath(_) | CtrlError::EFVCNotFound(_)
        _ => Err(AppError::NotFound.into()),
    }
}

// TODO this should NOT be a full-on server function, but should be something
// that is only available on the server side, with dedicated client functions
// for each specific response type.  This is only acceptable in a prototype.
// FIXME remove server macro
#[server(ReadBlob, "/api", output = Rkyv)]
pub async fn read_blob(
    id: i64,
    path: String,
    efvid: i64,
    key: String,
) -> Result<Vec<u8>, AppError> {
    session().await?
        .enforcer(format!("/exposure/{id}/"), "").await?;
    let platform = platform().await?;
    let ec = platform.get_exposure(id).await
        .map_err(|_| AppError::InternalServerError)?;
    let efc = ec.ctrl_path(&path).await
        .map_err(|_| AppError::InternalServerError)?;
    let efvc = efc.get_view(efvid).await
        .map_err(|_| AppError::InternalServerError)?;
    Ok(efvc.read_blob(&key)
        .await
        .map_err(|_| AppError::InternalServerError)?)
}

// for now restricting this to just the `index.html`.
#[server(ReadSafeIndexHtml, "/api", output = Rkyv)]
pub async fn read_safe_index_html(
    id: i64,
    path: String,
    efvid: i64,
) -> Result<String, AppError> {
    fn evaluate(url: &str) -> Option<Cow<'_, str>> {
        match url.as_bytes() {
            [b'/', ..] => Some(url.into()),
            _ => Some(["../", url].concat().into()),
        }
    }

    session().await?
        .enforcer(format!("/exposure/{id}/"), "").await?;

    let blob = read_blob(id, path.clone(), efvid, "index.html".to_string())
        .await
        .map_err(|_| AppError::InternalServerError)?;
    Ok(Builder::new()
        .url_relative(UrlRelative::Custom(Box::new(evaluate)))
        .clean(&String::from_utf8_lossy(&blob))
        .to_string()
    )
}

#[cfg_attr(feature = "utoipa", utoipa::path(
    post,
    path = "/api/create_exposure",
    request_body(
        description = r#"
Attempt to resolve additional information about a path within an exposure.
        "#,
        content((
            Id = "application/json",
            examples(
                ("Example 1" = (
                    summary = "Create the new exposure by the alias and commit id.",
                    value = json!({
                        "id": {
                            "Aliased": "beeler_reuter_1977",
                        },
                        "commit_id": "cb090c96a2ce627457b14def4910ac39219b8340",
                    })
                )),
            )
        )),
    ),
    responses((
        status = 200,
        description = "Path to the new exposure",
        body = String,
        example = "/exposure/123/",
    ), AppError),
    security(
        ("cookie" = []),
    ),
))]
#[server(
    endpoint = "create_exposure",
    input = server_fn::codec::Json,
)]
pub async fn create_exposure_openapi(
    id: Id,
    commit_id: String,
) -> Result<String, AppError> {
    let workspace_id = crate::server::workspace::resolve_id(id).await?;
    Ok(create_exposure_core(workspace_id, commit_id).await?)
}

#[server]
pub async fn create_exposure(
    workspace_id: i64,
    commit_id: String,
) -> Result<(), AppError> {
    let target = create_exposure_core(workspace_id, commit_id).await?;
    leptos_axum::redirect(&target);
    Ok(())
}

#[cfg(feature = "ssr")]
async fn create_exposure_core(
    workspace_id: i64,
    commit_id: String,
) -> Result<String, AppError> {
    let policy_state = session().await?
        .enforcer_and_policy_state("/exposure/", "create").await?;
    // also validate workspace read permissions, as having the ability to create
    // exposures in general does not imply read permission on that workspace.
    session().await?
        .enforcer(format!("/workspace/{workspace_id}/"), "").await?;
    let platform = platform().await?;
    // First create the workspace
    let ctrl = platform.create_exposure(
        workspace_id,
        &commit_id,
    )
        .await
        .map_err(|_| AppError::InternalServerError)?;

    // then set the default workflow state to private
    let id = ctrl.exposure().id();
    let resource = format!("/exposure/{id}/");
    platform
        .ac_platform
        .set_wf_state_for_res(&resource, State::Private)
        .await
        .map_err(|_| AppError::InternalServerError)?;
    // and grant the current user the owner permission

    if let Some(policy) = policy_state.policy {
        if let Agent::User(user) = policy.agent {
            platform
                .ac_platform
                .res_grant_role_to_agent(&resource, user, Role::Owner)
                .await
                .map_err(|_| AppError::InternalServerError)?;
        }
    }

    let alias = ctrl.allocate_alias().await
        .map_err(|_| AppError::InternalServerError)?;

    Ok(format!("/exposure/{alias}"))
}

#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Serialize, Deserialize)]
pub struct WizardInfo {
    pub exposure: Exposure,
    pub files: Vec<(String, Option<(ExposureFileProfile, UserPromptGroups)>)>,
    pub profiles: Vec<Profile>,
}

#[cfg_attr(feature = "utoipa", utoipa::path(
    post,
    path = "/api/exposure_wizard",
    request_body(
        description = r#"
Acquire `WizardInfo` for the given exposure.
        "#,
        content((
            Id = "application/json",
            examples(
                ("Example 1" = (
                    summary = "Acquire the information by the exposure's alias.",
                    value = json!({
                        "id": {
                            "Aliased": "c1",
                        },
                    }),
                )),
            )
        )),
    ),
    responses((
        status = 200,
        description = "Wizard information within an `EnforcedOk`.",
        body = EnforcedOk<WizardInfo>,
    ), AppError),
    security(
        ("cookie" = []),
    ),
))]
#[server(
    endpoint = "exposure_wizard",
    input = server_fn::codec::Json,
)]
pub async fn wizard(
    id: Id,
) -> Result<EnforcedOk<WizardInfo>, AppError> {
    let id = resolve_id(id).await?;
    let policy_state = session().await?
        .enforcer_and_policy_state(format!("/exposure/{id}/"), "edit").await?;
    let platform = platform().await?;
    let ctrl = platform.get_exposure(id).await
        .map_err(|_| AppError::InternalServerError)?;
    let prompts = ctrl.pair_files_profile_prompt_groups().await
        .map_err(log_error)?;
    let profiles = platform.list_profiles().await
        .map_err(|_| AppError::InternalServerError)?;

    let exposure = ctrl.exposure().clone_inner();
    let files = prompts.into_iter()
        .map(|(path, value)| (
            path.to_owned(),
            value.map(|(profile, upg)| (
                profile,
                upg.to_owned(),
            ))
        ))
        .collect::<Vec<_>>();

    Ok(policy_state.to_enforced_ok(WizardInfo {
        exposure,
        files,
        profiles,
    }))
}

// this struct is a placeholder to help utoipa
#[cfg(feature = "utoipa")]
#[allow(dead_code)]
#[derive(utoipa::ToSchema)]
struct WizardAddFileArgs {
    id: Id,
    path: String,
    profile_id: i64,
}

#[cfg_attr(feature = "utoipa", utoipa::path(
    post,
    path = "/api/exposure_wizard_add_file",
    request_body(
        description = r#"
Add a file to a wizard
        "#,
        content((
            WizardAddFileArgs = "application/json",
            examples(
                ("Example 1" = (
                    summary = "Acquire the information by the exposure's alias.",
                    value = json!({
                        "id": {
                            "Aliased": "c1",
                        },
                        "path": "beeler_reuter_1977.cellml",
                        "profile_id": 1,
                    }),
                )),
            )
        )),
    ),
    responses((
        status = 200,
        description = "Denotes success",
        body = EnforcedOk<WizardInfo>,
    ), AppError),
    security(
        ("cookie" = []),
    ),
))]
#[server(
    endpoint = "exposure_wizard_add_file",
    input = server_fn::codec::Json,
)]
pub async fn wizard_add_file_openapi(
    id: Id,
    path: String,
    profile_id: i64,
) -> Result<EnforcedOk<WizardInfo>, AppError> {
    let exposure_id = resolve_id(id.clone()).await?;
    wizard_add_file(exposure_id, path, profile_id).await?;
    wizard(id).await
}

#[server]
pub async fn wizard_add_file(
    exposure_id: i64,
    path: String,
    profile_id: i64,
) -> Result<(), AppError> {
    session().await?
        .enforcer(format!("/exposure/{exposure_id}/"), "edit").await?;
    let platform = platform().await?;
    let ec = platform.get_exposure(exposure_id).await
        .map_err(|_| AppError::InternalServerError)?;
    let efc = ec.create_file(&path).await
        .map_err(|_| AppError::InternalServerError)?;
    let vtt_profile = platform.get_view_task_template_profile(profile_id).await
        .map_err(|_| AppError::InternalServerError)?;
    efc.set_vttprofile(vtt_profile).await
        .map_err(|_| AppError::InternalServerError)?;
    Ok(())
}

#[cfg(feature = "ssr")]
pub async fn update_wizard_field(
    _fields: Vec<(String, String)>,
) -> Result<(), AppError> {
    Ok(())
}

#[cfg(not(feature = "ssr"))]
pub fn update_wizard_field(
    fields: Vec<(String, String)>,
) -> impl std::future::Future<Output = Result<(), AppError>> + Send + 'static {
    use http::status::StatusCode;
    use leptos::prelude::on_cleanup;
    use send_wrapper::SendWrapper;
    use wasm_bindgen::UnwrapThrowExt;
    use web_sys::FormData;

    SendWrapper::new(async move {
        let abort_controller =
            SendWrapper::new(web_sys::AbortController::new().ok());
        let abort_signal = abort_controller.as_ref().map(|a| a.signal());

        // abort in-flight requests if, e.g., we've navigated away from this page
        on_cleanup(move || {
            if let Some(abort_controller) = abort_controller.take() {
                abort_controller.abort()
            }
        });

        let form_data = FormData::new()
            .expect("web_sys::FormData must be available for use");
        for (name, value) in fields.iter() {
            let _ = form_data.append_with_str(name, value)
                .map_err(|e| leptos::logging::error!("{e:?}"));
        }
	let params =
	    web_sys::UrlSearchParams::new_with_str_sequence_sequence(
		&form_data,
	    )
	    .unwrap_throw();

        let resp = leptos::server_fn::request::browser::Request::post(WIZARD_FIELD_ROUTE)
            .abort_signal(abort_signal.as_ref())
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(params)
            .map_err(|_| AppError::InternalServerError)?
            .send()
            .await
            .map_err(|e| {
                leptos::logging::error!("E: {e}");
                AppError::InternalServerError
            })?;

        if resp.ok() {
            // TODO check whether or not the field actually got updated
            Ok(())
        } else {
            Err(AppError::from(
                StatusCode::from_u16(resp.status())
                    .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
            ))
        }
    })
}

#[cfg_attr(feature = "utoipa", utoipa::path(
    post,
    path = "/api/exposure_wizard_build",
    request_body(
        description = r#"
Build the exposure.
        "#,
        content((
            WizardAddFileArgs = "application/json",
            examples(
                ("Example 1" = (
                    summary = "Specify the exposure to build.",
                    value = json!({
                        "id": {
                            "Aliased": "c1",
                        },
                    }),
                )),
            )
        )),
    ),
    responses((
        status = 200,
        description = "Number of tasks queued.",
        body = usize,
    ), AppError),
    security(
        ("cookie" = []),
    ),
))]
#[server(
    endpoint = "exposure_wizard_build",
    input = server_fn::codec::Json,
)]
pub async fn wizard_build_openapi(
    id: Id,
) -> Result<usize, AppError> {
    let exposure_id = resolve_id(id).await?;
    wizard_build(exposure_id).await
}

#[server]
pub async fn wizard_build(
    exposure_id: i64,
) -> Result<usize, AppError> {
    session().await?
        .enforcer(format!("/exposure/{exposure_id}/"), "edit").await?;
    let platform = platform().await?;
    let result = platform.process_vttc_tasks_for_exposure(exposure_id).await
        .map_err(|_| AppError::InternalServerError)?;
    Ok(result)
}
