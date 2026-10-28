use axum::{
    routing::get,
    response::Redirect,
    Router,
};
use camino::Utf8PathBuf;
use clap::Parser;
use pmr2_gateway::GatewayBuilder;
use std::net::SocketAddr;

#[derive(Debug, Parser)]
struct Cli {
    // TODO somehow automatically generate this uri if VHM is used, e.g. if bind_addr is `localhost:4140` it
    // should generate: "http://127.0.0.1:8280/VirtualHostBase/http/localhost:4140/pmr/VirtualHostRoot/",
    // however this may be more trouble than it's actually worth.

    /// The base uri for the PMR2 instance.
    #[clap(
        value_name = "PMR2_PROXY_HOST_BASE",
        env = "PMR2_PROXY_HOST_BASE",
    )]
    uri_base: String,

    /// Directory to cache the PMR2 data.
    #[clap(
        long,
        value_name = "PMR2_PROXY_CACHE_DIR",
        env = "PMR2_PROXY_CACHE_DIR",
    )]
    cache_dir: Utf8PathBuf,

    /// The number of concurrent active connections permitted to access the underlying PMR2 for the
    /// resource restricted endpoints.
    #[clap(
        short = 'p',
        long,
        value_name = "PMR2_PROXY_PERMITS",
        env = "PMR2_PROXY_PERMITS",
        default_value_t = 4,
    )]
    permits: usize,
    #[clap(
        long,
        value_name = "PMR2_PROXY_BIND_ADDR",
        env = "PMR2_PROXY_BIND_ADDR",
    )]
    bind_addr: SocketAddr,
}


#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    let Cli { uri_base, cache_dir, permits, bind_addr } = Cli::parse();

    let service = GatewayBuilder::new()
        .cache_dir(cache_dir)
        .cache_permits(permits)
        .passthru_permits(permits)
        .build_for(uri_base)
        .unwrap();

    let mut router = Router::new()
        // While this could use the service, in PMR2 this is setup to redirect to some specific location,
        // so just go do that.  For now default this to `/welcome`, but they do go to separate targets
        // depending on the domain this got accessed from.
        // .route_service("/", service.clone())
        // This optimization avoids a connection to the underlying PMR2 instance.
        .route("/", get(|| async { Redirect::permanent("/welcome") }))
        .route_service("/{*path}", service);

    #[cfg(feature = "trace")]
    {
        use tower_http::trace::TraceLayer;
        router = router.layer(TraceLayer::new_for_http());
        tracing_subscriber::fmt::init();
    }

    let listener = tokio::net::TcpListener::bind(bind_addr).await.unwrap();
    println!("listening on {bind_addr}");
    axum::serve(listener, router.into_make_service())
        .await
        .unwrap();
}
