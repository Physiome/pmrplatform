use clap::Parser;
use pmrmodel::backend::db::SqliteBackend;
use pmrapp_server::config::Config;
use pmrapp_server::http;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    stderrlog::new()
        .module(module_path!())
        .verbosity(2)
        .timestamp(stderrlog::Timestamp::Second)
        .init()
        .unwrap();
    let config = Config::parse();
    let backend = SqliteBackend::from_url(&config.pmrapp_db_url).await?;
    http::serve(config, backend).await;
    Ok(())
}
