use clap::Parser;
use pmrac::platform::Builder as ACPlatformBuilder;
use pmrcore::error::BackendError;
use pmrctrl::{
    error::PlatformError,
    executor::Executor,
    platform::Platform,
};
use pmrmodel::backend::db::{
    MigrationProfile,
    SqliteBackend,
};
use pmrtqs::runner::RunnerRuntime;
use sqlx::{
    Sqlite,
    migrate::MigrateDatabase,
};
use std::fs;
use tokio;

#[derive(Debug, Parser)]
struct Cli {
    #[clap(short = 'r', long = "runners", default_value = "8")]
    runners: usize,
    #[clap(long, value_name = "PMR_DATA_ROOT", env = "PMR_DATA_ROOT")]
    pmr_data_root: String,
    #[clap(long, value_name = "PMR_REPO_ROOT", env = "PMR_REPO_ROOT")]
    pmr_repo_root: String,
    #[clap(long, value_name = "PMRAC_DB_URL", env = "PMRAC_DB_URL")]
    pmrac_db_url: String,
    #[clap(long, value_name = "PMRAPP_DB_URL", env = "PMRAPP_DB_URL")]
    pmrapp_db_url: String,
    #[clap(long, value_name = "PMRTQS_DB_URL", env = "PMRTQS_DB_URL")]
    pmrtqs_db_url: String,
    #[clap(short = 'v', long = "verbose", action = clap::ArgAction::Count)]
    verbose: u8,
}


fn main() -> Result<(), PlatformError> {
    dotenvy::dotenv().ok();
    let args = Cli::parse();
    stderrlog::new()
        .module(module_path!())
        .module("pmrctrl")
        .module("pmrtqs")
        .verbosity((args.verbose as usize) + 1)
        .timestamp(stderrlog::Timestamp::Second)
        .init()
        .unwrap();

    let rt = tokio::runtime::Runtime::new().unwrap();
    let platform = rt.block_on(async {
        if !Sqlite::database_exists(&args.pmrac_db_url).await.unwrap_or(false) {
            log::warn!("pmrac database {} does not exist; creating...", &args.pmrac_db_url);
            Sqlite::create_database(&args.pmrac_db_url).await
                .map_err(BackendError::from)?
        }

        if !Sqlite::database_exists(&args.pmrapp_db_url).await.unwrap_or(false) {
            log::warn!("pmrapp database {} does not exist; creating...", &args.pmrapp_db_url);
            Sqlite::create_database(&args.pmrapp_db_url).await
                .map_err(BackendError::from)?
        }
        if !Sqlite::database_exists(&args.pmrtqs_db_url).await.unwrap_or(false) {
            log::warn!("pmrtqs database {} does not exist; creating...", &args.pmrtqs_db_url);
            Sqlite::create_database(&args.pmrtqs_db_url).await
                .map_err(BackendError::from)?
        }
        let ac = SqliteBackend::from_url(&args.pmrac_db_url)
            .await
            .map_err(BackendError::from)?
            .run_migration_profile(MigrationProfile::Pmrac)
            .await
            .map_err(BackendError::from)?;
        let mc = SqliteBackend::from_url(&args.pmrapp_db_url)
            .await
            .map_err(BackendError::from)?
            .run_migration_profile(MigrationProfile::Pmrapp)
            .await
            .map_err(BackendError::from)?;
        let tm = SqliteBackend::from_url(&args.pmrtqs_db_url)
            .await
            .map_err(BackendError::from)?
            .run_migration_profile(MigrationProfile::Pmrtqs)
            .await
            .map_err(BackendError::from)?;
        let platform = Platform::new(
            ACPlatformBuilder::new()
                .ac_platform(ac)
                .build(),
            mc,
            tm,
            fs::canonicalize(&args.pmr_data_root)?,
            fs::canonicalize(&args.pmr_repo_root)?,
        );
        Ok::<_, PlatformError>(platform)
    })?;
    let executor = Executor::new(platform);
    let mut runtime = RunnerRuntime::new(executor, args.runners);
    runtime.start();
    log::info!("runner runtime starting");
    runtime.wait();
    log::info!("runner runtime stopped");
    Ok(())
}
