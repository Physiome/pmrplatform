#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    use axum::{
        Router,
        extract::Extension,
        routing::get,
    };
    use axum_login::AuthManagerLayerBuilder;
    use clap::Parser;
    use leptos::prelude::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use pmrac::platform::Builder as ACPlatformBuilder;
    use pmrapp::app::*;
    use pmrapp::conf::Cli;
    use pmrapp::server::workspace::raw_workspace_download;
    use pmrctrl::platform::Platform;
    use pmrmodel::backend::db::{
        MigrationProfile,
        SqliteBackend,
    };
    use std::fs;
    use sqlx::{migrate::MigrateDatabase, Sqlite};
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
    dbg!(&routes);

    if !Sqlite::database_exists(&args.pmrac_db_url).await.unwrap_or(false) {
        warn!("pmrac database {} does not exist; creating...", &args.pmrac_db_url);
        Sqlite::create_database(&args.pmrac_db_url).await?;
    }
    if !Sqlite::database_exists(&args.pmrapp_db_url).await.unwrap_or(false) {
        warn!("pmrapp database {} does not exist; creating...", &args.pmrapp_db_url);
        Sqlite::create_database(&args.pmrapp_db_url).await?;
    }
    if !Sqlite::database_exists(&args.pmrtqs_db_url).await.unwrap_or(false) {
        warn!("pmrtqs database {} does not exist; creating...", &args.pmrtqs_db_url);
        Sqlite::create_database(&args.pmrtqs_db_url).await?;
    }
    let ac = SqliteBackend::from_url(&args.pmrac_db_url)
        .await?
        .run_migration_profile(MigrationProfile::Pmrac)
        .await?;
    let mc = SqliteBackend::from_url(&args.pmrapp_db_url)
        .await?
        .run_migration_profile(MigrationProfile::Pmrapp)
        .await?;
    let tm = SqliteBackend::from_url(&args.pmrtqs_db_url)
        .await?
        .run_migration_profile(MigrationProfile::Pmrtqs)
        .await?;
    let platform = Platform::new(
        ACPlatformBuilder::new()
            .ac_platform(ac)
            .build(),
        mc,
        tm,
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
        .route("/workspace/:workspace_id/rawfile/:commit_id/*path",
            get(raw_workspace_download)
        )
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

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    log!("listening on http://{}", &addr);
    axum::serve(listener, app.into_make_service())
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
