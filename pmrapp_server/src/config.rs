#[derive(clap::Parser)]
pub struct Config {
    /// The database URL
    #[clap(long, env)]
    pub pmrapp_db_url: String,

    /// The root for the git repos
    #[clap(long, env)]
    pub pmr_git_root: String,

    /// The http port for the server
    #[clap(long, env)]
    pub http_port: u16,
}
