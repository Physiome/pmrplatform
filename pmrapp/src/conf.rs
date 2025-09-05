use clap::Parser;
use pmrctrl::platform::Builder;

#[derive(Debug, Parser)]
pub struct Cli {
    #[clap(flatten)]
    pub platform_builder: Builder,
    #[clap(short = 'v', long = "verbose", action = clap::ArgAction::Count)]
    pub verbose: u8,
    #[clap(long, default_value = "0")]
    pub with_runners: usize,
}
