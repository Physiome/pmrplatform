#[tokio::main]
async fn main() -> anyhow::Result<()> {
    use axum::{
        Router,
        ServiceExt,
        http::header::HeaderValue,
    };
    use clap::Parser;
    use pmrapp::{
        integration::PmrAxumExt,
        conf::Cli,
    };
    use pmrctrl::executor::Executor;
    use pmrtqs::runtime::Builder as RuntimeBuilder;

    dotenvy::dotenv().ok();
    let args = Cli::parse();

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
        .pmr_layers(
            platform.clone(),
            Some(args.cors_allow_origins
                .iter()
                .map(String::as_ref)
                .map(str::parse::<HeaderValue>)
                .collect::<Result<Vec<_>, _>>()?
                .into()),
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
