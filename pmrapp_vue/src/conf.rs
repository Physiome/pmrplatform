use clap::Parser;
use pmrapp::conf;
use std::path::PathBuf;

#[derive(Debug, Parser)]
pub struct PmrappVueConf {
    #[clap(long, value_name = "VUE_ASSET_PATH", env = "VUE_ASSET_PATHS", default_value = "asset")]
    pub vue_asset_path: PathBuf,
    #[clap(long, value_name = "VUE_UPDATE_SCRIPT", env = "VUE_UPDATE_SCRIPT")]
    pub vue_update_script: Option<PathBuf>,
    #[clap(long, value_name = "VUE_UPDATE_BEARER", env = "VUE_UPDATE_BEARER")]
    pub vue_update_bearer: Option<String>,
    #[clap(flatten)]
    pub pmrapp_args: conf::Cli,
}
