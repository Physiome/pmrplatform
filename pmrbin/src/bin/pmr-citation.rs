use clap::{Parser, Subcommand};
use pmrcore::{
    exposure::{
        traits::{
            Exposure as _,
            ExposureFile as _,
            ExposureFileView as _,
        },
        profile::traits::ExposureFileProfileBackend,
    },
};
use pmrctrl::platform::{
    Builder as PlatformBuilder,
    Platform,
};
use pmrmodel::{
    model::{
        profile::UserViewProfileRef,
        task_template::TaskArgBuilder,
    },
};
use std::{
    fs,
    io::{
        stdin,
        BufReader,
    },
    sync::OnceLock,
};

#[derive(Debug, Parser)]
struct Config {
    #[clap(short = 'v', long = "verbose", action = clap::ArgAction::Count)]
    verbose: u8,
}

static CONF: OnceLock<Config> = OnceLock::new();

#[derive(Debug, Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    #[clap(flatten)]
    platform_builder: PlatformBuilder,
    #[clap(flatten)]
    config: Config,
}

#[derive(Debug, Subcommand)]
enum Commands {
    #[command(arg_required_else_help = true)]
    Add {
        identifier: String,
    },
    #[command(arg_required_else_help = true)]
    Link {
        identifier: String,
        resource_path: String,
    },
    #[command(arg_required_else_help = true)]
    Rdfxml {
        #[command(subcommand)]
        cmd: RdfxmlCmd,
    },
}

#[derive(Debug, Subcommand)]
enum RdfxmlCmd {
    #[command(arg_required_else_help = true)]
    Index {
        path: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    let args = Cli::parse();
    stderrlog::new()
        .module(module_path!())
        .module("pmrdb")
        .verbosity((args.config.verbose as usize) + 1)
        .timestamp(stderrlog::Timestamp::Second)
        .init()
        .unwrap();

    let platform = args.platform_builder
        .build()
        .await
        .map_err(anyhow::Error::from_boxed)?;

    let _ = CONF.set(args.config);

    match args.command {
        Commands::Add { .. } => {
            todo!()
        },
        Commands::Link { .. } => {
            todo!()
        },
        Commands::Rdfxml { cmd } => {
            parse_rdfxml_cmd(&platform, cmd).await?;
        },
    }

    Ok(())
}

async fn parse_rdfxml_cmd<'p>(
    platform: &'p Platform,
    arg: RdfxmlCmd,
) -> anyhow::Result<()> {
    match arg {
        RdfxmlCmd::Index { .. } => {
            todo!()
        }
    }
    Ok(())
}
