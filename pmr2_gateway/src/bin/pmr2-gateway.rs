use axum::{
    extract::Request,
    http::HeaderValue,
    routing::get,
    response::Redirect,
    Router,
};
use clap::Parser;
use pmr2_gateway::Cli;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    let cli = Cli::parse();
    let service = cli.to_gateway();

    let mut router = Router::new()
        // While this could use the service, in PMR2 this is setup to redirect to some specific location,
        // so just go do that.  For now default this to `/welcome`, but they do go to separate targets
        // depending on the domain this got accessed from.
        // .route_service("/", service.clone())
        // This optimization avoids a connection to the underlying PMR2 instance.
        .route("/", get(|request: Request| async move {
            match request.headers().get("x-forwarded-host") {
                Some(s) if s == HeaderValue::from_static("models.cellml.org") =>
                    Redirect::permanent("/cellml"),
                Some(s) if s == HeaderValue::from_static("models.fieldml.org") =>
                    Redirect::permanent("/fieldml"),
                _ => Redirect::permanent("/welcome"),
            }
        }))
        .route_service("/{*path}", service);

    #[cfg(feature = "trace")]
    {
        use tower_http::trace::TraceLayer;
        router = router.layer(TraceLayer::new_for_http());
        tracing_subscriber::fmt::init();
    }

    let bind_addr = cli.bind_addr;
    let listener = tokio::net::TcpListener::bind(bind_addr).await.unwrap();
    println!("listening on {bind_addr}");
    axum::serve(listener, router.into_make_service())
        .await
        .unwrap();
}
