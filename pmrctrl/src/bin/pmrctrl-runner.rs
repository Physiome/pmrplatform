use clap::Parser;
use pmrac::platform::Builder as ACPlatformBuilder;
use pmrctrl::{
    executor::Executor,
    platform::Platform,
};
use pmrdb::{
    Backend,
    ConnectorOption,
};
use pmrtqs::runtime::Runtime;
use std::{
    error::Error,
    fs,
};
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


fn main() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
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
        let platform = Platform::new(
            ACPlatformBuilder::new()
                .boxed_ac_platform(
                    Backend::ac(
                        ConnectorOption::from(&args.pmrac_db_url)
                            .auto_create_db(true)
                    )
                        .await?,
                )
                .build(),
            Backend::mc(
                ConnectorOption::from(&args.pmrapp_db_url)
                    .auto_create_db(true)
            )
                .await?
                .into(),
            Backend::tm(
                ConnectorOption::from(&args.pmrtqs_db_url)
                    .auto_create_db(true)
            )
                .await?
                .into(),
            fs::canonicalize(args.pmr_data_root)?,
            fs::canonicalize(args.pmr_repo_root)?,
        );
        Ok::<_, Box<dyn Error + Send + Sync + 'static>>(platform)
    })?;
    let executor = Executor::new(platform);
    let mut runtime = Runtime::new(executor, args.runners);
    runtime.start();
    log::info!("runner runtime starting");
    runtime.wait();
    log::info!("runner runtime stopped");
    Ok(())
}
