#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    use axum::{
        Router,
        extract::Extension,
        routing::get,
    };
    use clap::Parser;
    use leptos::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use pmrapp::app::*;
    use pmrapp::conf::Cli;
    use pmrapp::fileserv::file_and_error_handler;
    use pmrapp::server::workspace::raw_workspace_download;
    use pmrctrl::platform::Platform;
    use pmrmodel::backend::db::{
        MigrationProfile,
        SqliteBackend,
    };
    use std::fs;
    use sqlx::{migrate::MigrateDatabase, Sqlite};

    dotenvy::dotenv().ok();
    let args = Cli::parse();
    // Setting get_configuration(None) means we'll be using cargo-leptos's env values
    // For deployment these variables are:
    // <https://github.com/leptos-rs/start-axum#executing-a-server-on-a-remote-machine-without-the-toolchain>
    // Alternately a file can be specified such as Some("Cargo.toml")
    // The file would need to be included with the executable when moved to deployment
    let conf = get_configuration(None).await.unwrap();
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr;
    let routes = generate_route_list(App);
    dbg!(&routes);

    if !Sqlite::database_exists(&args.pmrapp_db_url).await.unwrap_or(false) {
        logging::warn!("pmrapp database {} does not exist; creating...", &args.pmrapp_db_url);
        Sqlite::create_database(&args.pmrapp_db_url).await?;
    }
    if !Sqlite::database_exists(&args.pmrtqs_db_url).await.unwrap_or(false) {
        logging::warn!("pmrtqs database {} does not exist; creating...", &args.pmrtqs_db_url);
        Sqlite::create_database(&args.pmrtqs_db_url).await?;
    }
    let mc = SqliteBackend::from_url(&args.pmrapp_db_url)
        .await?
        .run_migration_profile(MigrationProfile::Pmrapp)
        .await?;
    let tm = SqliteBackend::from_url(&args.pmrtqs_db_url)
        .await?
        .run_migration_profile(MigrationProfile::Pmrtqs)
        .await?;
    let platform = Platform::new(
        mc,
        tm,
        fs::canonicalize(&args.pmr_data_root)?,
        fs::canonicalize(&args.pmr_repo_root)?,
    );

    // build our application with a route
    let app = Router::new()
        .route("/workspace/:workspace_id/raw/:commit_id/*path",
            get(raw_workspace_download::<SqliteBackend, SqliteBackend>)
        )
        .leptos_routes(
            &leptos_options,
            routes,
            App,
        )
        .fallback(file_and_error_handler)
        .layer(Extension(platform.clone()))
        .with_state(leptos_options);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    logging::log!("listening on http://{}", &addr);
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
