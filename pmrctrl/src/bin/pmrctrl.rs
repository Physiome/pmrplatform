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
    profile::traits::{
        ProfileBackend,
        ProfileViewsBackend,
        ViewTaskTemplateBackend,
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
    #[command(arg_required_else_help = true)]
    Profile {
        #[command(subcommand)]
        cmd: ProfileCmd,
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

#[derive(Debug, Subcommand)]
enum ProfileCmd {
    #[command(arg_required_else_help = true)]
    Create {
        title: String,
        description: String,
    },
    #[command(arg_required_else_help = true)]
    View {
        profile_id: i64,
    },
    #[command(arg_required_else_help = true)]
    Vtt {
        profile_id: i64,
        task_template_id: i64,
        view_key: String,
        description: String,
    },
    // TODO dump a profile to JSON?
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
        Commands::Profile { cmd } => {
            parse_profile(cmd, &platform).await?;
        },
        // TODO exposure (view template) profiles
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
        },
        // FileCmd::View { exposure_file_id, effv_id } => {
        //     todo!();
        // },
    }
    Ok(())
}

async fn parse_profile<'p, MCP, TMP>(
    arg: ProfileCmd,
    platform: &'p Platform<MCP, TMP>,
) -> anyhow::Result<()>
where
    MCP: MCPlatform + Sized + Send + Sync,
    TMP: TMPlatform + Sized + Send + Sync,
{
    match arg {
        ProfileCmd::Create { title, description } => {
            let backend: &dyn ProfileBackend = platform.mc_platform.as_ref();
            let id = backend.insert_profile(
                &title,
                &description,
            ).await?;
            println!("created new profile id: {id}");
        },
        ProfileCmd::View { profile_id } => {
            let result = platform.get_view_task_template_profile(profile_id).await?;
            let output = serde_json::to_string_pretty(&result)?;
            println!("{output}");
        },
        ProfileCmd::Vtt { profile_id, task_template_id, view_key, description } => {
            let backend: &dyn ViewTaskTemplateBackend = platform.mc_platform.as_ref();
            let id = backend.insert_view_task_template(
                &view_key,
                &description,
                task_template_id,
            ).await?;
            println!("created under profile id: {profile_id} a new exposure file view task template id: {id} with view_key {view_key} using template {task_template_id}");
            let pvb: &dyn ProfileViewsBackend = platform.mc_platform.as_ref();
            pvb.insert_profile_views(profile_id, id).await?;

            let vtt = platform.get_view_task_template(id).await?;
            let output = serde_json::to_string_pretty(&vtt)?;
            println!("{output}");
        },
    }
    Ok(())
}
