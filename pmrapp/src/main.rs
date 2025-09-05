#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    use axum::{
        Router,
        ServiceExt,
        extract::{
            Extension,
            Request,
        },
        http::{
            Uri,
            header::HeaderValue,
            uri::PathAndQuery,
        },
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
    use pmrapp::server::workspace::{
        collection_json_workspace,
        raw_aliased_workspace_download,
        raw_workspace_download,
    };
    use pmrapp::server::exposure::{
        exposure_file_data,
        wizard_field_update,
    };
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
    use tower::{
        Layer,
        ServiceBuilder,
        util::MapRequestLayer,
    };
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

    let platform = args.platform_builder.build().await
        .map_err(anyhow::Error::from_boxed)?;

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
        // TODO the path should be constructed from a known list, so that rewriting only happens
        // to this route only if it exists.
        .route("/collection_json/workspace/", get(collection_json_workspace))
        .route("/data/exposure/{e_id}/{ef_id}/{view_key}/{*path}", get(exposure_file_data))
        .route("/workspace/{workspace_alias}/rawfile/{commit_id}/{*path}", get(raw_aliased_workspace_download))
        .route("/workspace/:/id/{workspace_id}/rawfile/{commit_id}/{*path}", get(raw_workspace_download))
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

    fn reroute_collection_json<B: std::fmt::Debug>(mut req: Request<B>) -> Request<B> {
        // naively resolve our header
        if req.headers().get("accept") == Some(&HeaderValue::from_static("application/vnd.physiome.pmr2.json.1")) {
            // TODO this should be defined as a constant for use with building the router
            let prefix = "/collection_json";
            let mut parts = req.uri().clone().into_parts();
            parts.path_and_query = parts.path_and_query
                .map(|v| PathAndQuery::try_from(format!("{prefix}{v}")).expect("original parsed fine"));
            *req.uri_mut() = Uri::from_parts(parts).expect("original parts should be valid");
        }
        req
    }

    let middleware = MapRequestLayer::new(reroute_collection_json);

    let app = middleware.layer(app);

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
