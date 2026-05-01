use clap::Parser;
use pmrmeta::cli::{Cli, Config};
use std::sync::OnceLock;

static CONF: OnceLock<Config> = OnceLock::new();

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    let args = Cli::parse();
    stderrlog::new()
        .module(module_path!())
        .module("pmrdb")
        .module("pmrmeta")
        .verbosity((args.config.verbose as usize) + 1)
        .timestamp(stderrlog::Timestamp::Second)
        .init()
        .unwrap();

    let platform = args.platform_builder
        .build()
        .await
        .map_err(anyhow::Error::from_boxed)?;

    let _ = CONF.set(args.config);

    args.command.run(platform).await?;

    Ok(())
}
