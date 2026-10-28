use axum::{
    extract::Request,
    http::HeaderValue,
    routing::get,
    response::Redirect,
    Router,
};
use clap::Parser;
use pmr2_gateway::Cli;
use tower_http::trace::TraceLayer;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    let router = Router::new()
        // As this gateway is intended to handle the two other alternate hosts, this will need the appropriate
        // redirect that used to be done by the underlying vhost handler in Plone.  Incidentally this also
        // serves as an optimization to avoid a connection to that underlying instance.
        .route("/", get(|request: Request| async move {
            match request.headers().get("x-forwarded-host") {
                Some(s) if s == HeaderValue::from_static("models.cellml.org") =>
                    Redirect::permanent("/cellml"),
                Some(s) if s == HeaderValue::from_static("models.fieldml.org") =>
                    Redirect::permanent("/fieldml"),
                _ => Redirect::permanent("/welcome"),
            }
        }))
        .route_service("/{*path}", cli.gateway())
        .layer(cli.rewrite_pmr2_layer())
        .layer(TraceLayer::new_for_http());

    let bind_addr = cli.bind_addr;
    let listener = tokio::net::TcpListener::bind(bind_addr).await.unwrap();
    tracing::info!("listening on {bind_addr}");
    axum::serve(listener, router.into_make_service())
        .await
        .unwrap();
}
