use clap::Parser;
use pmrapp::conf;
use std::path::PathBuf;

#[derive(Debug, Parser)]
pub struct PmrappVueConf {
    #[clap(long, value_name = "VUE_ASSET_PATH", env = "VUE_ASSET_PATHS", default_value = "asset")]
    pub vue_asset_path: PathBuf,
    #[clap(flatten)]
    pub pmrapp_args: conf::Cli,
}
