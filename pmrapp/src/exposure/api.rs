use leptos::{
    prelude::ServerFnError,
    server,
    server_fn::codec::Rkyv,
};
use pmrcore::{
    exposure::{
        Exposure,
        Exposures,
        profile::ExposureFileProfile,
    },
    profile::{
        Profile,
        UserPromptGroups,
    },
    task_template::UserArgs,
    workspace::Workspace,
};
use serde::{Serialize, Deserialize};
use crate::{
    enforcement::EnforcedOk,
    error::AppError,
};
use super::ResolvedExposurePath;

#[cfg(feature = "ssr")]
mod ssr {
    pub use ammonia::{
        Builder,
        UrlRelative,
    };
    pub use axum::http::request::Parts;
    pub use leptos::prelude::expect_context;
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

#[server]
pub async fn list() -> Result<EnforcedOk<Exposures>, ServerFnError<AppError>> {
    let policy_state = session().await?
        .enforcer_and_policy_state("/exposure/", "").await?;
    let platform = platform().await?;
    Ok(policy_state.to_enforced_ok(ExposureBackend::list(platform.mc_platform.as_ref())
        .await
        .map_err(|_| AppError::InternalServerError)?))
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ExposureInfo {
    pub exposure: Exposure,
    pub files: Vec<(String, bool)>,
    pub workspace: Workspace,
}

#[server]
pub async fn get_exposure_info(id: i64) -> Result<EnforcedOk<ExposureInfo>, ServerFnError<AppError>> {
    let policy_state = session().await?
        .enforcer_and_policy_state(format!("/exposure/{id}/"), "").await?;
    let platform = platform().await?;
    let ctrl = platform.get_exposure(id).await
        .map_err(|_| AppError::InternalServerError)?;
    let files = ctrl.list_files_info().await
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
) -> Result<EnforcedOk<ResolvedExposurePath>, ServerFnError<AppError>> {
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
            //
            // Returning the path as an Ok(Err(..)) to indicate a proper
            // result that isn't a ServerFnError, but still an inner Err
            // to facilitate this custom redirect handling.
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
) -> Result<Box<[u8]>, ServerFnError<AppError>> {
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
) -> Result<String, ServerFnError<AppError>> {
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
    id: i64,
    commit_id: String,
) -> Result<(), ServerFnError<AppError>> {
    let policy_state = session().await?
        .enforcer_and_policy_state("/exposure/", "create").await?;
    let platform = platform().await?;
    // First create the workspace
    let ctrl = platform.create_exposure(
        id,
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

    leptos_axum::redirect(format!("/exposure/{id}").as_ref());
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
    id: i64,
) -> Result<EnforcedOk<WizardInfo>, ServerFnError<AppError>> {
    let policy_state = session().await?
        .enforcer_and_policy_state(format!("/exposure/{id}/"), "edit").await?;
    let platform = platform().await?;
    let ctrl = platform.get_exposure(id).await
        .map_err(|_| AppError::InternalServerError)?;
    let prompts = ctrl.list_files_profile_prompt_groups().await
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
) -> Result<(), ServerFnError<AppError>> {
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
