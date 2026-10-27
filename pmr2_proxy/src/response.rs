use axum::{
    body::Body,
    http::{header, Response, StatusCode},
};
use crate::template::message;

pub(crate) fn http_error(status: StatusCode, msg: &str) -> Response<Body> {
    let msg = message(&format!(
        "<h1>{} {}</h1><p>{msg}</p>",
        status.as_str(),
        status.canonical_reason().unwrap_or(""),
    ))
    .expect("input shouldn't fail to parse and processing shouldn't fail here");
    Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, "text/html")
        .body(Body::from(msg))
        .expect("statically constructed response shouldn't error")
}

// pub(crate) fn http_404() -> Response<Body> {
//     http_error(StatusCode::NOT_FOUND, "");
// }

pub(crate) fn http_404() -> Response<Body> {
    const PLONE_404_MSG: &str = r#"
<h1 class="documentFirstHeading">
  This page does not seem to exist…
</h1>
<div id="content-core">
  <p>
    We apologize for the inconvenience, but the page you were trying to access is not at this address.
    You can use the links below to help you find what you are looking for.
  </p>
  <p>
    If you are certain you have the correct web address but are encountering an error, please
    contact the <span> <a href="/contact-info">Site Administration</a></span>.
  </p>
  <p>Thank you.</p>
</div>
"#;
    let msg = message(PLONE_404_MSG)
        .expect("input shouldn't fail to parse and processing shouldn't fail here");
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .header(header::CONTENT_TYPE, "text/html")
        .body(Body::from(msg))
        .expect("statically constructed response shouldn't error")
}

pub(crate) fn http_500() -> Response<Body> {
    http_error(
        StatusCode::INTERNAL_SERVER_ERROR,
        r#"
        The Gateway service for the Physiome Model Repository has encountered an unexpected error. If you wish,
        please contact the <span><a href="/contact-info">Site Administration</a></span>.
        "#,
    )
}

pub(crate) fn http_503() -> Response<Body> {
    http_error(
        StatusCode::SERVICE_UNAVAILABLE,
        r#"
        The Physiome Model Repository is currently experiencing an unexpected downtime, please try again later.
        "#,
    )
}

pub(crate) fn http_504() -> Response<Body> {
    http_error(
        StatusCode::GATEWAY_TIMEOUT,
        r#"
        The Physiome Model Repository is currently experiencing an excessively high volume of requests, please
        try again later.
        "#,
    )
}
