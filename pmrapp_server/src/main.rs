use clap::Parser;
use pmrmodel::backend::db::SqliteBackend;
use server::config::Config;
use server::http;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    stderrlog::new()
        .module(module_path!())
        .verbosity(2)
        .timestamp(stderrlog::Timestamp::Second)
        .init()
        .unwrap();
    let config = Config::parse();
    let backend = SqliteBackend::from_url(&config.database_url).await?;
    http::serve(config, backend).await?;
    Ok(())
}
