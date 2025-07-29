use clap::Parser;
use pmrcore::platform::TMPlatform;
use pmrdb::{
    Backend,
    ConnectorOption,
};
use pmrtqs::{
    executor::TMPlatformExecutor,
    runtime::Runtime,
};
use std::sync::Arc;
use tokio;

// the runner may become a separate sync function
// start the long running command
// - it will wait for 1 second? at a time
// - check the cancellation token, if not cancelled, keep going
//   - if cancelled, reduce life to 10 seconds.
// - repeat 600 times (5 minutes?)
// - kill the command if still running after that

#[derive(Debug, Parser)]
struct Cli {
    #[clap(short = 'r', long = "runners", default_value = "8")]
    runners: usize,
    #[clap(long, value_name = "PMR_DATA_ROOT", env = "PMR_DATA_ROOT")]
    pmr_data_root: String,
    #[clap(long, value_name = "PMR_REPO_ROOT", env = "PMR_REPO_ROOT")]
    pmr_repo_root: String,
    #[clap(long, value_name = "PMRAPP_DB_URL", env = "PMRAPP_DB_URL")]
    pmrapp_db_url: String,
    #[clap(long, value_name = "PMRTQS_DB_URL", env = "PMRTQS_DB_URL")]
    pmrtqs_db_url: String,
    #[clap(short = 'v', long = "verbose", action = clap::ArgAction::Count)]
    verbose: u8,
}


fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    let args = Cli::parse();
    stderrlog::new()
        .module(module_path!())
        .module("pmrtqs")
        .verbosity((args.verbose as usize) + 1)
        .timestamp(stderrlog::Timestamp::Second)
        .init()
        .unwrap();

    let rt = tokio::runtime::Runtime::new().unwrap();
    let backend = rt.block_on(async {
        Ok::<_, anyhow::Error>(Backend::tm(ConnectorOption::from(&args.pmrtqs_db_url).auto_create_db(true))
            .await
            .map_err(anyhow::Error::from_boxed)?
        )
    })?;
    let executor = TMPlatformExecutor::new(<Arc<dyn TMPlatform>>::from(backend));
    let mut runtime = Runtime::new(executor.clone(), args.runners);
    runtime.start();
    log::info!("runner runtime starting");
    runtime.wait();
    log::info!("runner runtime stopped");
    Ok(())
}
