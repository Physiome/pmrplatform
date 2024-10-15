use clap::Parser;

#[derive(Debug, Parser)]
pub struct Cli {
    #[clap(long, value_name = "PMR_DATA_ROOT", env = "PMR_DATA_ROOT")]
    pub pmr_data_root: String,
    #[clap(long, value_name = "PMR_REPO_ROOT", env = "PMR_REPO_ROOT")]
    pub pmr_repo_root: String,
    #[clap(long, value_name = "PMRAC_DB_URL", env = "PMRAC_DB_URL")]
    pub pmrac_db_url: String,
    #[clap(long, value_name = "PMRAPP_DB_URL", env = "PMRAPP_DB_URL")]
    pub pmrapp_db_url: String,
    #[clap(long, value_name = "PMRTQS_DB_URL", env = "PMRTQS_DB_URL")]
    pub pmrtqs_db_url: String,
    #[clap(short = 'v', long = "verbose", action = clap::ArgAction::Count)]
    pub verbose: u8,
}
