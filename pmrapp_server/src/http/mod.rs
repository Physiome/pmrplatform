use axum::{
    extract::Extension,
    http::header::{
        HeaderMap,
        HeaderName,
        HeaderValue,
    },
    Json,
    response::{
        Html,
        IntoResponse,
        Response,
    },
    routing::get,
    Router,
};
use pmrmodel::backend::db::SqliteBackend;
use serde::Serialize;
use std::{
    net::SocketAddr,
    sync::Arc,
};
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;

use crate::config::Config;
pub use pmrapp_client::sauron;
use pmrapp_client::App;
use pmrapp_client::sauron::Render;

mod page;
mod error;
mod api;
mod workspace;

pub use error::Error;

pub type Result<T, E = Error> = std::result::Result<T, E>;


// TODO figure out if this can actually be pub
#[derive(Clone)]
pub struct AppContext {
    config: Arc<Config>,
    backend: SqliteBackend,
}


pub async fn serve(config: Config, backend: SqliteBackend) -> anyhow::Result<()> {
    let socket: SocketAddr = ([0, 0, 0, 0], config.http_port).into();
    let app = router()
        .nest("/workspace/", workspace::router())
        .nest("/api/workspace/", api::workspace::router())
        .layer(
            ServiceBuilder::new()
            .layer(Extension(AppContext {
                config: Arc::new(config),
                backend: backend,
            }))
            .layer(TraceLayer::new_for_http())
        );

    println!("serving at: http://{}", socket);
    axum::Server::bind(&socket)
        .serve(app.into_make_service())
        .await
        .unwrap();
    Ok(())
}


fn router() -> Router {
    Router::new()
        .route("/", get(index_root))
        .route("/api/", get(api_root))
        .route("/style/main.css", get(style_main))
        .route("/pkg/pmrapp_client.js", get(client_js))
        .route("/pkg/pmrapp_client_bg.wasm", get(client_bg_wasm))
}

// placeholder thingers
async fn index_root() -> Response {
    let app = App::with_homepage();
    let content = page::index(&app).render_to_string();
    Html(content).into_response()
}

#[derive(Serialize)]
struct Page {
    name: String,
}

async fn api_root() -> Response {
    let resp = Page { name: "index".to_string() };
    Json(resp).into_response()
}

async fn style_main() -> (HeaderMap, String) {
    let mut headers = HeaderMap::new();
    headers.insert(HeaderName::from_static("content-type"),
        HeaderValue::from_static("text/css"));
    (headers, page::style())
}

async fn client_js() -> (HeaderMap, String) {
    let mut headers = HeaderMap::new();
    headers.insert(HeaderName::from_static("content-type"),
        HeaderValue::from_static("text/javascript"));

    (headers, include_str!("../../../pmrapp_client/pkg/pmrapp_client.js").to_string())
}

async fn client_bg_wasm() -> (HeaderMap, Vec<u8>) {
    let mut headers = HeaderMap::new();
    headers.insert(HeaderName::from_static("content-type"),
        HeaderValue::from_static("application/wasm"));

    (headers, include_bytes!("../../../pmrapp_client/pkg/pmrapp_client_bg.wasm").to_vec())
}
