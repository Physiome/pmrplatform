use clap::{Parser, Subcommand};
use pmrctrl::platform::Builder as PlatformBuilder;
use pmrctrl::platform::Platform;

mod docgen;
mod cmeta;

use docgen::Docgen;

#[derive(Debug, Parser)]
pub struct Arguments {
    #[clap(long)]
    pub input_path: String,
    #[clap(long)]
    pub output_dir: String,
    #[clap(long)]
    pub exposure_id: i64,
    #[clap(long)]
    pub exposure_path: String,
}

#[derive(Debug, Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
    #[clap(flatten)]
    pub platform_builder: PlatformBuilder,
    #[clap(flatten)]
    pub config: Config,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    #[command(arg_required_else_help = true)]
    Cmeta(Arguments),
    #[command(arg_required_else_help = true)]
    Docgen {
        #[command(subcommand)]
        docgen: Docgen,
    },
}

#[derive(Debug, Parser)]
pub struct Config {
    #[clap(short = 'v', long = "verbose", action = clap::ArgAction::Count)]
    pub verbose: u8,
}


impl Commands {
    pub async fn run(self, platform: Platform) -> anyhow::Result<()> {
        match self {
            Commands::Docgen { docgen } => {
                docgen.run().await?;
            }
            Commands::Cmeta(arguments) => {
                cmeta::run(&platform, arguments).await?;
            }
        }
        Ok(())
    }
}
