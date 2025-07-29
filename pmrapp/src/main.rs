#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    use axum::{
        Router,
        extract::Extension,
        routing::{
            get,
            post,
        },
    };
    use axum_login::AuthManagerLayerBuilder;
    use clap::Parser;
    use leptos::prelude::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use pmrac::platform::Builder as ACPlatformBuilder;
    use pmrapp::app::*;
    use pmrapp::conf::Cli;
    use pmrapp::exposure::api::WIZARD_FIELD_ROUTE;
    use pmrapp::server::workspace::raw_workspace_download;
    use pmrapp::server::exposure::wizard_field_update;
    use pmrctrl::{
        executor::Executor,
        platform::Platform,
    };
    use pmrdb::{
        Backend,
        ConnectorOption,
    };
    use pmrrbac::Builder as PmrRbacBuilder;
    use pmrtqs::runtime::Builder as RuntimeBuilder;
    use std::fs;
    use time::Duration;
    use tower::ServiceBuilder;
    use tower_sessions::{Expiry, MemoryStore, SessionManagerLayer};

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

    // Setting get_configuration(None) means we'll be using cargo-leptos's env values
    // For deployment these variables are:
    // <https://github.com/leptos-rs/start-axum#executing-a-server-on-a-remote-machine-without-the-toolchain>
    // Alternately a file can be specified such as Some("Cargo.toml")
    // The file would need to be included with the executable when moved to deployment
    let conf = get_configuration(None).unwrap();
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr;
    let routes = generate_route_list(App);
    log::trace!("{routes:?}");

    let platform = Platform::new(
        ACPlatformBuilder::new()
            .boxed_ac_platform(
                Backend::ac(
                    ConnectorOption::from(&args.pmrac_db_url)
                        .auto_create_db(true)
                )
                    .await
                    .map_err(anyhow::Error::from_boxed)?,
            )
            .pmrrbac_builder(
                PmrRbacBuilder::new()
                    .anonymous_reader(true)
            )
            .build(),
        Backend::mc(
            ConnectorOption::from(&args.pmrapp_db_url)
                .auto_create_db(true)
        )
            .await
            .map_err(anyhow::Error::from_boxed)?
            .into(),
        Backend::tm(
            ConnectorOption::from(&args.pmrtqs_db_url)
                .auto_create_db(true)
        )
            .await
            .map_err(anyhow::Error::from_boxed)?
            .into(),
        fs::canonicalize(&args.pmr_data_root)?,
        fs::canonicalize(&args.pmr_repo_root)?,
    );

    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false)
        .with_expiry(Expiry::OnInactivity(Duration::days(1)));

    let auth_service = ServiceBuilder::new()
        .layer(
            AuthManagerLayerBuilder::new(
                platform.ac_platform.clone(),
                session_layer,
            ).build()
        );

    // build our application with a route
    let app = Router::new()
        .without_v07_checks()
        .route("/workspace/{workspace_id}/rawfile/{commit_id}/{*path}", get(raw_workspace_download))
        .route(WIZARD_FIELD_ROUTE, post(wizard_field_update))
        .leptos_routes(
            &leptos_options,
            routes,
            {
                let leptos_options = leptos_options.clone();
                move || shell(leptos_options.clone())
            },
        )
        .fallback(leptos_axum::file_and_error_handler(shell))
        // TODO add an additional handler that will filter out the body
        // for status code 3xx to optimize output.
        .layer(Extension(platform.clone()))
        .layer(auth_service)
        .with_state(leptos_options);

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

#[cfg(not(feature = "ssr"))]
pub fn main() {
    // no client-side main function
    // unless we want this to work with e.g., Trunk for a purely client-side app
    // see lib.rs for hydration function instead
}
