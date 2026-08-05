use clap::Parser;
use pmrctrl::platform::Builder;
use std::net::SocketAddr;

#[derive(Debug, Parser)]
pub struct Cli {
    #[clap(long, value_name = "CORS_ALLOW_ORIGIN", env = "CORS_ALLOW_ORIGIN", value_delimiter = ';')]
    pub cors_allow_origins: Vec<String>,
    #[clap(flatten)]
    pub platform_builder: Builder,
    #[clap(short = 'v', long = "verbose", action = clap::ArgAction::Count)]
    pub verbose: u8,
    #[clap(long, default_value = "0")]
    pub with_runners: usize,
    #[clap(long, value_name = "BIND_ADDR", env = "BIND_ADDR")]
    pub bind_addr: SocketAddr,
}
