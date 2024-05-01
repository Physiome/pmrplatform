use clap::{
    Parser,
    Subcommand,
};
use pmrcore::{
    exposure::{
        traits::{
            Exposure as _,
            ExposureFile as _,
        },
    },
    platform::{
        MCPlatform,
        TMPlatform,
    },
    profile::traits::{
        ProfileBackend,
        ProfileViewsBackend,
        ViewTaskTemplateBackend,
        ViewTaskTemplateProfileBackend,
    },
};
use pmrctrl::platform::Platform;
use pmrmodel::{
    backend::db::{
        Profile,
        SqliteBackend,
    },
    model::profile::UserViewProfileRef,
    registry::{
        ChoiceRegistry,
        ChoiceRegistryCache,
        PreparedChoiceRegistry,
    },
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
    #[command(arg_required_else_help = true)]
    Files {
        id: i64,
    },
    #[command(arg_required_else_help = true)]
    Path {
        exposure_id: i64,
        path: String,
        #[command(subcommand)]
        cmd: ExposurePathCmd,
    },
}

#[derive(Debug, Subcommand)]
enum FileCmd {
    #[command(arg_required_else_help = true)]
    Add {
        exposure_id: i64,
        path: String,
        profile_id: i64,
    },
    #[command(arg_required_else_help = true)]
    Profile {
        exposure_file_id: i64,
        #[command(subcommand)]
        cmd: FileProfileCmd,
    }
}

#[derive(Debug, Subcommand)]
enum ProfileCmd {
    #[command(arg_required_else_help = true)]
    Create {
        title: String,
        description: String,
    },
    #[command(arg_required_else_help = true)]
    Update {
        id: i64,
        title: String,
        description: String,
    },
    #[command(arg_required_else_help = true)]
    View {
        #[clap(long, short='p', action)]
        as_prompts: bool,
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

#[derive(Debug, Subcommand)]
enum FileProfileCmd {
    #[command(arg_required_else_help = true)]
    Assign {
        profile_id: i64,
    },
    // #[command(arg_required_else_help = true)]
    // Prompts,
}

#[derive(Debug, Subcommand)]
enum ExposurePathCmd {
    Prompts,
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
            parse_exposure(&platform, cmd).await?;
        },
        Commands::File { cmd } => {
            parse_file(&platform, cmd).await?;
        },
        Commands::Profile { cmd } => {
            parse_profile(&platform, cmd).await?;
        },
        // TODO exposure (view template) profiles
    }

    Ok(())
}

async fn parse_exposure<'p, MCP, TMP>(
    platform: &'p Platform<MCP, TMP>,
    arg: ExposureCmd,
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
        ExposureCmd::Files { id } => {
            let ctrl = platform.get_exposure(id).await?;
            let mut files = ctrl.list_files()?;
            files.sort_unstable();
            let mut exposure_files = ctrl.list_exposure_files().await?;
            exposure_files.sort_unstable();
            let mut exposure_files = exposure_files.into_iter().peekable();
            for file in files.iter() {
                if exposure_files.peek() == Some(&(file.as_ref())) {
                    println!("[*] {file}");
                    exposure_files.next();
                } else {
                    println!("[ ] {file}");
                }
            }
        }
        ExposureCmd::Path { exposure_id, path, cmd } => {
            parse_exposure_path(&platform, exposure_id, path.as_ref(), cmd).await?;
        }
    }
    Ok(())
}

async fn parse_file<'p, MCP, TMP>(
    platform: &'p Platform<MCP, TMP>,
    arg: FileCmd,
) -> anyhow::Result<()>
where
    MCP: MCPlatform + Sized + Send + Sync,
    TMP: TMPlatform + Sized + Send + Sync,
{
    match arg {
        FileCmd::Add { exposure_id, path, profile_id } => {
            let ec = platform.get_exposure(exposure_id).await?;
            let efc = ec.create_file(&path).await?;
            let id = efc.exposure_file().id();
            println!("created exposure file id {id} for exposure {exposure_id} at path {path}");
            let vtt_profile = ViewTaskTemplateProfileBackend::get_view_task_template_profile(
                platform.mc_platform.as_ref(),
                profile_id,
            ).await?;
            // TODO figure out if the ctrl platform should have a helper
            // that will initialize the profile_id for exposure_file_id
            // via the ExposureFileProfileBackend.
            platform.mc_platform.set_ef_vttprofile(
                id,
                vtt_profile,
            ).await?;
            println!("set exposure file id {id} with profile id {profile_id}");
        },
        FileCmd::Profile { exposure_file_id, cmd } => {
            parse_file_profile(&platform, exposure_file_id, cmd).await?;
        },
    }
    Ok(())
}

async fn parse_profile<'p, MCP, TMP>(
    platform: &'p Platform<MCP, TMP>,
    arg: ProfileCmd,
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
        ProfileCmd::Update { id, title, description } => {
            let backend: &dyn ProfileBackend = platform.mc_platform.as_ref();
            backend.update_profile_by_fields(
                id,
                &title,
                &description,
            ).await?;
            println!("updated profile {id}");
        },
        ProfileCmd::View { as_prompts, profile_id } => {
            let result = platform.get_view_task_template_profile(profile_id).await?;
            let output = if as_prompts {
                let registry = PreparedChoiceRegistry::new();
                let cache = ChoiceRegistryCache::from(
                    &registry as &dyn ChoiceRegistry<_>);
                let uvpr: UserViewProfileRef = (&result, &cache).into();
                serde_json::to_string_pretty(&uvpr)?
            } else {
                serde_json::to_string_pretty(&result)?
            };
            println!("{output}");
        },
        // profile vtt assign [x]
        // profile vtt remove [ ]
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

async fn parse_file_profile<'p, MCP, TMP>(
    platform: &'p Platform<MCP, TMP>,
    exposure_file_id: i64,
    arg: FileProfileCmd,
) -> anyhow::Result<()>
where
    MCP: MCPlatform + Sized + Send + Sync,
    TMP: TMPlatform + Sized + Send + Sync,
{
    match arg {
        FileProfileCmd::Assign { profile_id } => {
            let vttp = platform.get_view_task_template_profile(profile_id).await?;
            platform.mc_platform.set_ef_vttprofile(
                exposure_file_id,
                vttp,
            ).await?;
            println!("profile set: exposure_file_id {exposure_file_id} => profile_id {profile_id}");
        },
        // FileProfileCmd::Prompts => {
        //     println!("prompts for exposure_file_id {exposure_file_id}");
        //     // Still haven't figured out how to return this directly from
        //     // the platform, though with the typical usage via URI/paths
        //     // this isn't the same huge problem.
        //     let efc = todo!();
        //     let efvttsc = efc.build_vttc().await?;
        // },
    }
    Ok(())
}

async fn parse_exposure_path<'p, MCP, TMP>(
    platform: &'p Platform<MCP, TMP>,
    exposure_id: i64,
    path: &str,
    arg: ExposurePathCmd,
) -> anyhow::Result<()>
where
    MCP: MCPlatform + Sized + Send + Sync,
    TMP: TMPlatform + Sized + Send + Sync,
{
    match arg {
        ExposurePathCmd::Prompts => {
            let ec = platform.get_exposure(exposure_id).await?;
            let efc = ec.ctrl_path(path).await?;
            let efvttsc = efc.build_vttc().await?;
            let upgr = efvttsc.create_user_prompt_groups().await?;
            let output = serde_json::to_string_pretty(&upgr)?;
            println!("{output}");
        },
    }
    Ok(())
}