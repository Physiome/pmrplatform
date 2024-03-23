use clap::{
    Parser,
    Subcommand,
};
use pmrcore::{
    exposure::traits::{
        Exposure as _,
        ExposureFile as _,
    },
    platform::{
        MCPlatform,
        TMPlatform,
    },
};
use pmrctrl::platform::Platform;
use pmrmodel::backend::db::{
    Profile,
    SqliteBackend,
};
use sqlx::{
    Sqlite,
    migrate::MigrateDatabase,
};


#[derive(Debug, Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    #[clap(long, value_name = "PMR_DATA_ROOT", env = "PMR_DATA_ROOT")]
    pmr_data_root: String,
    #[clap(long, value_name = "PMR_REPO_ROOT", env = "PMR_REPO_ROOT")]
    pmr_repo_root: String,
    #[clap(long, value_name = "PMRAPP_DB_URL", env = "PMRAPP_DB_URL")]
    pmrapp_db_url: String,
    #[clap(long, value_name = "PMRTQS_DB_URL", env = "PMRTQS_DB_URL")]
    pmrtqs_db_url: String,
    #[clap(short = 'v', long = "verbose", action = clap::ArgAction::Count)]
    verbose: u8,
}


#[derive(Debug, Subcommand)]
enum Commands {
    #[command(arg_required_else_help = true)]
    Exposure {
        #[command(subcommand)]
        cmd: ExposureCmd,
    },
    #[command(arg_required_else_help = true)]
    File {
        #[command(subcommand)]
        cmd: FileCmd,
    },
}

#[derive(Debug, Subcommand)]
enum ExposureCmd {
    #[command(arg_required_else_help = true)]
    Add {
        workspace_id: i64,
        commit_id: String,
    },
}

#[derive(Debug, Subcommand)]
enum FileCmd {
    #[command(arg_required_else_help = true)]
    Add {
        exposure_id: i64,
        path: String,
    },
}

#[async_std::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    let args = Cli::parse();
    stderrlog::new()
        .module(module_path!())
        .verbosity((args.verbose as usize) + 1)
        .timestamp(stderrlog::Timestamp::Second)
        .init()
        .unwrap();

    if !Sqlite::database_exists(&args.pmrapp_db_url).await.unwrap_or(false) {
        log::warn!("pmrapp database {} does not exist; creating...", &args.pmrapp_db_url);
        Sqlite::create_database(&args.pmrapp_db_url).await?
    }

    if !Sqlite::database_exists(&args.pmrtqs_db_url).await.unwrap_or(false) {
        log::warn!("pmrtqs database {} does not exist; creating...", &args.pmrtqs_db_url);
        Sqlite::create_database(&args.pmrtqs_db_url).await?
    }

    let mc = SqliteBackend::from_url(&args.pmrapp_db_url)
        .await?
        .run_migration_profile(Profile::Pmrapp)
        .await?;
    let tm = SqliteBackend::from_url(&args.pmrtqs_db_url)
        .await?
        .run_migration_profile(Profile::Pmrtqs)
        .await?;

    let platform = Platform::new(
        mc,
        tm,
        args.pmr_data_root.into(),
        args.pmr_repo_root.into(),
    );

    match args.command {
        Commands::Exposure { cmd } => {
            parse_exposure(cmd, &platform).await?;
        },
        Commands::File { cmd } => {
            parse_file(cmd, &platform).await?;
        },
    }

    Ok(())
}

async fn parse_exposure<'p, MCP, TMP>(
    arg: ExposureCmd,
    platform: &'p Platform<MCP, TMP>,
) -> anyhow::Result<()>
where
    MCP: MCPlatform + Sized + Send + Sync,
    TMP: TMPlatform + Sized + Send + Sync,
{
    match arg {
        ExposureCmd::Add { workspace_id, commit_id } => {
            let ctrl = platform.create_exposure(workspace_id, &commit_id).await?;
            let id = ctrl.exposure().id();
            println!("created exposure id {id} for workspace {workspace_id} at commit {commit_id}");
        }
    }
    Ok(())
}

async fn parse_file<'p, MCP, TMP>(
    arg: FileCmd,
    platform: &'p Platform<MCP, TMP>,
) -> anyhow::Result<()>
where
    MCP: MCPlatform + Sized + Send + Sync,
    TMP: TMPlatform + Sized + Send + Sync,
{
    match arg {
        FileCmd::Add { exposure_id, path } => {
            let ec = platform.get_exposure(exposure_id).await?;
            let efc = ec.create_file(&path).await?;
            let id = efc.exposure_file().id();
            println!("created exposure file id {id} for exposure {exposure_id} at path {path}");
        }
    }
    Ok(())
}
