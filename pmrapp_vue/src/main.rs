use axum::{
    Router,
    ServiceExt,
    extract::Extension,
    http::{
        header::HeaderValue,
        StatusCode,
    },
};
use clap::Parser;
use pmrapp::integration::PmrAxumExt;
use pmrapp_vue::{
    conf::PmrappVueConf,
    route::PmrVueAxumExt,
};
use pmrctrl::executor::Executor;
use pmrtqs::runtime::Builder as RuntimeBuilder;
use tower_http::{
    services::{
        ServeDir,
        ServeFile,
    },
    set_status::SetStatus,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {

    dotenvy::dotenv().ok();
    let args = PmrappVueConf::parse();
    let vue_asset_path = args.vue_asset_path;
    let args = args.pmrapp_args;

    stderrlog::new()
        .module(module_path!())
        .module("pmrctrl")
        .module("pmrtqs")
        .module("pmrac")
        .module("pmrdb")
        .module("pmrrbac")
        .module("pmrtqs")
        // .module("axum_login")
        // .module("tower_sessions")
        // .module("tower_sessions_core")
        .verbosity((args.verbose as usize) + 1)
        .timestamp(stderrlog::Timestamp::Second)
        .init()
        .unwrap();

    let addr = args.bind_addr;
    let platform = args.platform_builder.build().await
        .map_err(anyhow::Error::from_boxed)?;
    let app = Router::new()
        .pmr_routes()
        .pmr_vue_routes(&vue_asset_path)
        .pmr_layers(
            platform.clone(),
            Some(args.cors_allow_origins
                .iter()
                .map(String::as_ref)
                .map(str::parse::<HeaderValue>)
                .collect::<Result<Vec<_>, _>>()?
                .into()),
        )
        .layer(Extension(ServeFile::new(vue_asset_path.join("index.html"))))
        .fallback_service(
            ServeDir::new(&vue_asset_path)
                .fallback(
                    SetStatus::new(
                        ServeFile::new(vue_asset_path.join("index.html")),
                        StatusCode::NOT_FOUND,
                    ),
                ),
        )
        .pmr_map_request();
    let runtime = (args.with_runners > 0).then(|| {
        let executor = Executor::new(platform.clone());
        let mut runtime = RuntimeBuilder::from(executor)
            .permits(args.with_runners)
            .build_with_handle(tokio::runtime::Handle::current());
        runtime.start();
        runtime
    });

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    log::info!("listening on http://{}", &addr);
    axum::serve(listener, app.into_make_service())
        .with_graceful_shutdown((move || {
            async {
                if let Some(runtime) = runtime{
                    runtime.shutdown_signal().await
                } else {
                    tokio::signal::ctrl_c()
                        .await
                        .expect("failed to install Ctrl+C handler");
                }
            }
        })())
        .await
        .unwrap();

    Ok(())
}
