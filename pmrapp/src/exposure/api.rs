use leptos::{
    server,
    server_fn::codec::Rkyv,
};
use pmrcore::{
    alias::AliasEntry,
    exposure::{
        Exposure,
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
use super::ResolvedExposurePath;

pub const WIZARD_FIELD_ROUTE: &'static str = "/api/exposure_wizard_field";

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
        },
    };
}
#[cfg(feature = "ssr")]
use self::ssr::*;

pub type Exposures = Vec<AliasEntry<Exposure>>;

#[server]
pub async fn list() -> Result<EnforcedOk<Exposures>, AppError> {
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

#[server]
pub async fn list_aliased() -> Result<EnforcedOk<Vec<AliasEntry<Exposure>>>, AppError> {
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

#[cfg(feature = "ssr")]
async fn resolve_id(id: Id) -> Result<i64, AppError> {
    Ok(match id {
        Id::Number(s) => s.parse().map_err(|_| AppError::NotFound)?,
        Id::Aliased(s) => platform()
            .await?
            .mc_platform
            .resolve_alias("exposure", &s)
            .await
            .map_err(|_| AppError::InternalServerError)?
            .ok_or(AppError::NotFound)?,
    })
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ExposureInfo {
    pub exposure: Exposure,
    pub files: Vec<(String, bool)>,
    pub workspace: Workspace,
}

#[server]
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
    Ok(policy_state.to_enforced_ok(ExposureInfo { exposure, files, workspace }))
}

#[server]
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
            let path = format!(
                "/workspace/{}/rawfile/{}/{}",
                exposure.workspace_id(),
                exposure.commit_id(),
                path,
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
) -> Result<Box<[u8]>, AppError> {
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
    fn evaluate(url: &str) -> Option<Cow<str>> {
        match url.as_bytes() {
            [b'/', ..] => Some(url.into()),
            _ => Some(["../", url].concat().into()),
        }
    }

    session().await?
        .enforcer(format!("/exposure/{id}/"), "").await?;

    let blob = read_blob(id, path.clone(), efvid, "index.html".to_string())
        .await
        .map_err(|_| AppError::InternalServerError)?
        .into_vec();
    Ok(Builder::new()
        .url_relative(UrlRelative::Custom(Box::new(evaluate)))
        .clean(&String::from_utf8_lossy(&blob))
        .to_string()
    )
}

#[server]
pub async fn create_exposure(
    workspace_id: i64,
    commit_id: String,
) -> Result<(), AppError> {
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

    leptos_axum::redirect(format!("/exposure/{alias}").as_ref());
    Ok(())
}

#[derive(Clone, Serialize, Deserialize)]
pub struct WizardInfo {
    pub exposure: Exposure,
    pub files: Vec<(String, Option<(ExposureFileProfile, UserPromptGroups)>)>,
    pub profiles: Vec<Profile>,
}

#[server]
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

#[server]
pub async fn wizard_build(
    exposure_id: i64,
) -> Result<usize, AppError> {
    session().await?
        .enforcer(format!("/exposure/{exposure_id}/"), "edit").await?;
    let mut result = 0;
    let platform = platform().await?;
    let ec = platform.get_exposure(exposure_id).await
        .map_err(|_| AppError::InternalServerError)?;

    let mut args = Vec::new();
    for efvttc in ec.list_files_efvttcs().await
        .map_err(|_| AppError::InternalServerError)?
    {
        let profile = efvttc
            .exposure_file_ctrl()
            .profile()
            .await
            .map_err(|_| AppError::InternalServerError)?;
        args.push((efvttc, profile));
    }

    for (efvttc, profile) in args.iter() {
        let vttc_tasks = efvttc.create_tasks_from_input(&profile.user_input)
            .map_err(|_| AppError::InternalServerError)?;
        result += efvttc
            .exposure_file_ctrl()
            .process_vttc_tasks(vttc_tasks).await
            .map_err(|_| AppError::InternalServerError)?
            .len();
    }
    Ok(result)
}
