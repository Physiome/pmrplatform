use axum::{
    Extension,
    response::IntoResponse,
};
use axum_login::AuthSession;
use itertools::Itertools;
use regex::Regex;
use pmrac::Platform as ACPlatform;
use pmrcore::task_template::UserInputMap;
use pmrctrl::platform::Platform;
use std::{
    collections::HashMap,
    sync::LazyLock,
};
use crate::{
    error::AppError,
    server::ac::Session,
};

static RE: LazyLock<Regex> = LazyLock::new(||
    Regex::new(r"^(\d+)-(\d+)$").unwrap());

pub async fn wizard_field_update(
    platform: Extension<Platform>,
    session: Extension<AuthSession<ACPlatform>>,
    body: String,
) -> Result<impl IntoResponse, AppError> {
    // Steps 1-3 is parsing
    // Step 4 is access validation
    // Step 5 is database update

    // 1. parse the incoming body and split the key-value pairs for keys
    //    that have two numbers separated by a `-` which denotes that
    //    these are to be treated as exposure_file_id + arg_id pairs for
    //    submission to the system, and other keys
    let (fields, others): (Vec<_>, Vec<_>) = serde_urlencoded::from_str::<Vec<(String, String)>>(&body)
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

    // 4. grab the exposure_id from the other keys and enforce access
    let exposure_id = others.get("exposure_id")
        .ok_or(AppError::BadRequest)?
        .parse::<i64>()
        .map_err(|_| AppError::BadRequest)?;
    Session::from(session)
        .enforcer(format!("/exposure/{exposure_id}/"), "edit").await?;

    // 5. process the fields by grouping them by exposure_file_id, while
    //    ensuring that the exposure_file_id are under the exposure to
    //    maintain the security invariant.

    let ec = platform.get_exposure(exposure_id).await
        .map_err(|_| AppError::InternalServerError)?;

    // 5.1 have a temporary holding every processed vttc handle and the
    //     user inputs associated with the exposure file underlying the
    //     vttc.
    let mut args = Vec::new();
    for (efid, values) in fields.into_iter()
        .into_group_map()
        .into_iter()
    {
        let efc = ec.ctrl_id(efid).await
            .map_err(|_| AppError::InternalServerError)?;
        let vttc = efc
            .try_into_vttc()
            .await
            .map_err(|_| AppError::InternalServerError)?;
        let user_input = values.into_iter()
            .collect::<UserInputMap>();
        args.push((vttc, user_input))
    }

    // 5.2 now update the database with the user inputs.
    for (vttc, user_input) in args.iter() {
        vttc.update_user_input(&user_input)
            .await
            // TODO need to distinguish user input error vs platform error
            .map_err(|_| AppError::InternalServerError)?;
    }

    Ok(())
}
