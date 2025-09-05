use clap::Parser;
use pmrctrl::{
    executor::Executor,
    platform::Builder,
};
use pmrtqs::runtime::Runtime;
use std::error::Error;
use tokio;

#[derive(Debug, Parser)]
struct Cli {
    #[clap(short = 'r', long = "runners", default_value = "8")]
    runners: usize,
    #[clap(flatten)]
    platform_builder: Builder,
    #[clap(short = 'v', long = "verbose", action = clap::ArgAction::Count)]
    verbose: u8,
}


fn main() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    dotenvy::dotenv().ok();
    let args = Cli::parse();
    stderrlog::new()
        .module(module_path!())
        .module("pmrdb")
        .module("pmrctrl")
        .module("pmrtqs")
        .verbosity((args.verbose as usize) + 1)
        .timestamp(stderrlog::Timestamp::Second)
        .init()
        .unwrap();

    let rt = tokio::runtime::Runtime::new().unwrap();
    let platform = rt.block_on(async {
        let platform = args.platform_builder.build().await?;
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
