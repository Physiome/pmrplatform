use axum::{
    Extension,
    extract::Path,
    http::{
        header::{CONTENT_TYPE, HeaderMap},
        HeaderValue,
    },
    response::IntoResponse,
};
use axum_login::AuthSession;
use itertools::Itertools;
use regex::Regex;
use pmrac::Platform as ACPlatform;
use pmrcore::task_template::UserInputMap;
use pmrctrl::platform::Platform;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::LazyLock,
};
use crate::{
    app::id::Id,
    error::AppError,
    exposure::api::WIZARD_FIELD_ROUTE,
    server::{
        self,
        ac::Session,
    },
};

static RE: LazyLock<Regex> = LazyLock::new(||
    Regex::new(r"^(\d+)-(\d+)$").unwrap());

pub async fn resolve_id(id: Id) -> Result<i64, AppError> {
    server::resolve_id("exposure", id).await
}

#[cfg_attr(feature = "utoipa", utoipa::path(
    post,
    path = WIZARD_FIELD_ROUTE,
    request_body(
        description = r#"
Update the user input mapping for the provided `ExposureFile`s.
        "#,
        content((
            WizardFieldUpdateArgs = "application/json",
            examples(
                ("Example 1" = (
                    summary = "Acquire the information by the exposure's alias.",
                    value = json!({
                        "id": {
                            "Aliased": "c1",
                        },
                        "ef_user_input": {
                            "1": {
                                "5": "baylor_hollingworth_chandler_2002_a.cellml",
                                "7": "Legacy CellML-tmpdoc",
                                "18": "CellML RDF Metadata",
                                "19": "Creative Commons - Attributions 3.0 Unported",
                            },
                            "2": {
                                "5": "baylor_hollingworth_chandler_2002_b.cellml",
                                "7": "Legacy CellML-tmpdoc",
                                "18": "CellML RDF Metadata",
                                "19": "Creative Commons - Attributions 3.0 Unported",
                            },
                        }
                    }),
                )),
            )
        )),
    ),
    responses((
        status = 200,
        description = "This status code means all fields updated correctly.",
        body = (),
    ), AppError),
))]
pub async fn wizard_field_update(
    platform: Extension<Platform>,
    session: Extension<AuthSession<ACPlatform>>,
    headers: HeaderMap,
    body: String,
) -> Result<impl IntoResponse, AppError> {
    match headers.get(CONTENT_TYPE) {
        Some(v) if v == "application/x-www-form-urlencoded".parse::<HeaderValue>().unwrap() => {
            let (exposure_id, ef_user_input) = parse_wizard_field_update_www_form_urlencode(&body)?;
            process_ef_user_input(
                platform,
                session,
                exposure_id,
                ef_user_input,
            ).await
        }
        Some(v) if v == "application/json".parse::<HeaderValue>().unwrap() => {
            let (exposure_id, ef_user_input) = parse_wizard_field_update_json(&platform.0, &body).await?;
            process_ef_user_input(
                platform,
                session,
                exposure_id,
                ef_user_input,
            ).await
        }
        _ => Err(AppError::BadRequest),
    }
}

fn parse_wizard_field_update_www_form_urlencode(
    body: &str,
) -> Result<(i64, impl Iterator<Item = (i64, Vec<(i64, String)>)>), AppError> {
    // 1. parse the incoming body and split the key-value pairs for keys
    //    that have two numbers separated by a `-` which denotes that
    //    these are to be treated as exposure_file_id + arg_id pairs for
    //    submission to the system, and other keys
    let (fields, others): (Vec<_>, Vec<_>) = serde_urlencoded::from_str::<Vec<(String, String)>>(body)
        .map_err(|_| AppError::BadRequest)?
        .into_iter()
        .partition(|(k, _)| RE.is_match(k));

    // 2. convert the fields into the arguments to be passed into the
    //    database
    let fields = fields.into_iter()
        .map(|(k, v)| {
            let (_, [exposure_file_id, arg_id]) = RE.captures(&k)
                .expect("this was matched earlier")
                .extract();
            // errors would be caused by the incoming number being too
            // long.
            Ok((exposure_file_id.parse::<i64>()?, (arg_id.parse::<i64>()?, v)))
        })
        .collect::<Result<Vec<_>, std::num::ParseIntError>>()
        .map_err(|_| AppError::BadRequest)?;

    // 3. collect the rest into a hashmap for lookup
    let others = others.into_iter()
        .collect::<HashMap<_, _>>();

    // 4. grab the exposure_id from the other keys.
    let exposure_id = others.get("exposure_id")
        .ok_or(AppError::BadRequest)?
        .parse::<i64>()
        .map_err(|_| AppError::BadRequest)?;

    // 5. process the fields by grouping them by exposure_file_id, while
    //    ensuring that the exposure_file_id are under the exposure to
    //    maintain the security invariant.
    let ef_user_input = fields.into_iter()
        .into_group_map()
        .into_iter();

    Ok((exposure_id, ef_user_input))
}

#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Serialize, Deserialize)]
pub struct WizardFieldUpdateArgs {
    pub id: Id,
    pub ef_user_input: HashMap<i64, HashMap<i64, String>>,
}

async fn parse_wizard_field_update_json(
    platform: &Platform,
    body: &str,
) -> Result<(i64, impl Iterator<Item = (i64, HashMap<i64, String>)>), AppError> {
    // 1. parse the incoming body and split the key-value pairs for keys
    //    that have two numbers separated by a `-` which denotes that
    //    these are to be treated as exposure_file_id + arg_id pairs for
    //    submission to the system, and other keys
    let WizardFieldUpdateArgs {
        id,
        ef_user_input,
    } = serde_json::from_str(body)
        .map_err(|_| AppError::BadRequest)?;
    let exposure_id = id.resolve(&platform, "exposure").await?;
    Ok((exposure_id, ef_user_input.into_iter()))
}

async fn process_ef_user_input(
    platform: Extension<Platform>,
    session: Extension<AuthSession<ACPlatform>>,
    exposure_id: i64,
    ef_user_input: impl Iterator<Item = (i64, impl IntoIterator<Item = (i64, String)>)>,
) -> Result<(), AppError> {
    // 0. validate permission
    Session::from(session)
        .enforcer(format!("/exposure/{exposure_id}/"), "edit").await?;

    let ec = platform.get_exposure(exposure_id).await
        .map_err(|_| AppError::InternalServerError)?;

    // 1. have a temporary holding every processed vttc handle and the
    //     user inputs associated with the exposure file underlying the
    //     vttc.
    let mut args = Vec::new();
    for (efid, values) in ef_user_input {
        let efc = ec.ctrl_id(efid).await
            .map_err(|_| AppError::InternalServerError)?;
        let vttc = efc
            .try_into_vttc()
            .await
            .map_err(|_| AppError::InternalServerError)?;

        // Convert `Vec<(i64, std::string::String)>` into `HashMap<i64, std::string::String>`
        // TODO this somehow couples, it's desirable to allow HashMap (i.e. JavaScript's object)
        // to be passed directly here, so this logic should be decoupled.
        let user_input = values.into_iter()
            .collect::<UserInputMap>();
        args.push((vttc, user_input))
    }

    // 2. now update the database with the user inputs.
    for (vttc, user_input) in args.iter() {
        vttc.update_user_input(&user_input)
            .await
            // TODO need to distinguish user input error vs platform error
            // TODO also verify that only unaccepted inputs get rejected while
            // *all* accepted/valid inputs are processed.
            .map_err(|_| AppError::InternalServerError)?;
    }

    // FIXME should report error if _none_ of the fields updated, to allow
    // the situation where a single field being updated can easily have the
    // error response to indicate failure to update that one field.

    Ok(())
}

pub async fn exposure_file_data(
    platform: Extension<Platform>,
    session: Extension<AuthSession<ACPlatform>>,
    Path((e_id, ef_id, view_key, path)): Path<(i64, i64, String, String)>,
) -> Result<Vec<u8>, AppError> {
    Session::from(session)
        .enforcer(format!("/exposure/{e_id}/"), "").await?;
    let ec = platform.get_exposure(e_id).await
        .map_err(|_| AppError::InternalServerError)?;
    ec.read_blob(ef_id, &view_key, &path).await
        .map_err(|_| AppError::NotFound)

}
