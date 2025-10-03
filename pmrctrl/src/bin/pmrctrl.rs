use clap::{
    Parser,
    Subcommand,
    ValueEnum,
};
use pmrcore::{
    exposure::{
        traits::{
            Exposure as _,
            ExposureFile as _,
            ExposureFileView as _,
        },
        profile::traits::ExposureFileProfileBackend,
    },
    profile::traits::{
        ProfileBackend,
        ProfileViewsBackend,
        ViewTaskTemplateBackend,
        ViewTaskTemplateProfileBackend,
    },
    task_template::{
        UserArgs,
        UserInputMap,
    },
};
use pmrctrl::platform::{
    Builder as PlatformBuilder,
    Platform,
};
use pmrmodel::{
    model::{
        profile::UserViewProfileRef,
        task_template::TaskArgBuilder,
    },
    registry::{
        ChoiceRegistry,
        ChoiceRegistryCache,
        PreparedChoiceRegistry,
    },
};
use std::{
    fs,
    io::{
        stdin,
        BufReader,
    },
    sync::OnceLock,
};

#[derive(Clone, Debug, ValueEnum)]
pub enum SerdeKind {
    Json,
    Toml,
}

mod display {
    use std::fmt::{Display, Formatter, Result};
    use super::SerdeKind;

    impl Display for SerdeKind {
        fn fmt(&self, f: &mut Formatter) -> Result {
            match self {
                Self::Json => "json".fmt(f),
                Self::Toml => "toml".fmt(f),
            }
        }
    }
}

impl SerdeKind {
    pub fn to_string<T>(&self, item: &T) -> Result<String, anyhow::Error>
    where
        T: serde::Serialize,
    {
        Ok(match self {
            Self::Json => serde_json::to_string_pretty(item)?,
            Self::Toml => toml::to_string(item)?,
        })
    }

    pub fn from_reader<R, T>(&self, rdr: R) -> Result<T, anyhow::Error>
    where
        R: std::io::Read,
        T: serde::de::DeserializeOwned,
    {
        Ok(match self {
            Self::Json => serde_json::from_reader(rdr)?,
            Self::Toml => toml::from_slice(&rdr.bytes().collect::<Result<Vec<_>, _>>()?)?,
        })
    }
}

#[derive(Debug, Parser)]
struct Config {
    #[clap(short = 'o', long = "format", default_value_t = SerdeKind::Json)]
    serde_kind: SerdeKind,
    #[clap(short = 'v', long = "verbose", action = clap::ArgAction::Count)]
    verbose: u8,
}

static CONF: OnceLock<Config> = OnceLock::new();

#[derive(Debug, Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    #[clap(flatten)]
    platform_builder: PlatformBuilder,
    #[clap(flatten)]
    config: Config,
}


#[derive(Debug, Subcommand)]
enum Commands {
    #[command(arg_required_else_help = true)]
    Alias {
        #[command(subcommand)]
        cmd: AliasCmd,
    },
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
    #[command(arg_required_else_help = true)]
    Task {
        #[command(subcommand)]
        cmd: TaskCmd,
    },
    #[command(arg_required_else_help = true)]
    Vtt {
        #[command(subcommand)]
        cmd: VttCmd,
    },
}

#[derive(Debug, Subcommand)]
enum AliasCmd {
    #[command(arg_required_else_help = true)]
    Add {
        kind: String,
        kind_id: i64,
        alias: String,
    },
    #[command(arg_required_else_help = true)]
    List {
        kind: String,
        kind_id: i64,
    },
    #[command(arg_required_else_help = true)]
    Resolve {
        kind: String,
        alias: String,
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
    List,
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
    Import {
        input: Option<std::path::PathBuf>,
    },
    #[command(arg_required_else_help = true)]
    Export {
        #[clap(long, short='x', action)]
        as_prompts: bool,
        profile_id: i64,
    },
    #[command(arg_required_else_help = true)]
    Assign {
        profile_id: i64,
        vtt_id: i64,
    },
    #[command(arg_required_else_help = true)]
    Remove {
        profile_id: i64,
        vtt_id: i64,
    },
}

#[derive(Debug, Subcommand)]
enum TaskCmd {
    ExecOneShot,
}

#[derive(Debug, Subcommand)]
enum VttCmd {
    Import {
        input: Option<std::path::PathBuf>,
    },
    #[command(arg_required_else_help = true)]
    Link {
        task_template_id: i64,
        view_key: String,
        description: String,
    },
    #[command(arg_required_else_help = true)]
    Export {
        id: i64,
    }
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
    #[command(arg_required_else_help = true)]
    Answer {
        arg_id: i64,
        answer: String,
    },
    Answers,
    #[command(arg_required_else_help = true)]
    Assign {
        profile_id: i64,
    },
    Prompts,
    Task {
        #[clap(long, short='p', action)]
        submit: bool,
    },
    Views,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    let args = Cli::parse();
    stderrlog::new()
        .module(module_path!())
        .module("pmrdb")
        .module("pmrtqs")
        .verbosity((args.config.verbose as usize) + 1)
        .timestamp(stderrlog::Timestamp::Second)
        .init()
        .unwrap();

    let platform = args.platform_builder
        .build()
        .await
        .map_err(anyhow::Error::from_boxed)?;

    let _ = CONF.set(args.config);

    match args.command {
        Commands::Alias { cmd } => {
            parse_alias(&platform, cmd).await?;
        },
        Commands::Exposure { cmd } => {
            parse_exposure(&platform, cmd).await?;
        },
        Commands::File { cmd } => {
            parse_file(&platform, cmd).await?;
        },
        Commands::Profile { cmd } => {
            parse_profile(&platform, cmd).await?;
        },
        Commands::Task { cmd } => {
            parse_task(&platform, cmd).await?;
        },
        Commands::Vtt { cmd } => {
            parse_vtt(&platform, cmd).await?;
        },
    }

    Ok(())
}

async fn parse_alias<'p>(
    platform: &'p Platform,
    arg: AliasCmd,
) -> anyhow::Result<()> {
    match arg {
        AliasCmd::Add { kind, kind_id, alias } => {
            platform.mc_platform.add_alias(&kind, kind_id, &alias).await?;
            println!("added alias {alias} pointing to {kind}/:/id/{kind_id}");
        }
        AliasCmd::List { kind, kind_id } => {
            let aliases = platform.mc_platform.get_aliases(&kind, kind_id).await?;
            println!("listing aliases for {kind}/:/id/{kind_id}");
            for alias in aliases {
                let alias = &alias.alias;
                println!("- {kind}/{alias}");
            }
        }
        AliasCmd::Resolve { kind, alias } => {
            if let Some(kind_id) = platform.mc_platform.resolve_alias(&kind, &alias).await? {
                println!("aliases for {kind}/{alias} points to {kind}/:/id/{kind_id}");
            } else {
                println!("no such aliases: {kind}/{alias}; assume it points to {kind}/:id/{alias}");
            }
        }
    }
    Ok(())
}

async fn parse_exposure<'p>(
    platform: &'p Platform,
    arg: ExposureCmd,
) -> anyhow::Result<()> {
    match arg {
        ExposureCmd::Add { workspace_id, commit_id } => {
            let ctrl = platform.create_exposure(workspace_id, &commit_id).await?;
            let id = ctrl.exposure().id();
            println!("created exposure id {id} for workspace {workspace_id} at commit {commit_id}");
        }
        ExposureCmd::Files { id } => {
            let ctrl = platform.get_exposure(id).await?;
            for (file, flag) in ctrl.pair_files_info().await?.iter() {
                let flag = flag.then_some("*").unwrap_or(" ");
                println!("[{flag}] {file}");
            }
        }
        ExposureCmd::Path { exposure_id, path, cmd } => {
            parse_exposure_path(&platform, exposure_id, path.as_ref(), cmd).await?;
        }
    }
    Ok(())
}

async fn parse_file<'p>(
    platform: &'p Platform,
    arg: FileCmd,
) -> anyhow::Result<()> {
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
            efc.set_vttprofile(vtt_profile).await?;
            println!("set exposure file id {id} with profile id {profile_id}");
        },
        FileCmd::Profile { exposure_file_id, cmd } => {
            parse_file_profile(&platform, exposure_file_id, cmd).await?;
        },
    }
    Ok(())
}

async fn parse_profile<'p>(
    platform: &'p Platform,
    arg: ProfileCmd,
) -> anyhow::Result<()> {
    let conf = CONF.get().expect("config is set by main");
    match arg {
        ProfileCmd::List => {
            let profiles = ProfileBackend::list_profiles(platform.mc_platform.as_ref()).await?;
            println!("id - title");
            for profile in profiles.into_iter() {
                println!("{} - {}", profile.id, profile.title);
            }
        }
        ProfileCmd::Create { title, description } => {
            let id = ProfileBackend::insert_profile(
                platform.mc_platform.as_ref(),
                &title,
                &description,
            ).await?;
            println!("created new profile id: {id}");
        },
        ProfileCmd::Update { id, title, description } => {
            ProfileBackend::update_profile_by_fields(
                platform.mc_platform.as_ref(),
                id,
                &title,
                &description,
            ).await?;
            println!("updated profile {id}");
        },
        ProfileCmd::Import { input } => {
            let id = platform.add_view_task_template_profile(
                match input {
                    Some(path) => conf.serde_kind.from_reader(BufReader::new(fs::File::open(path)?))?,
                    None => conf.serde_kind.from_reader(BufReader::new(stdin()))?,
                }
            ).await?;
            println!("imported ViewTaskTemplateProfile {id}");
        },
        ProfileCmd::Export { as_prompts, profile_id } => {
            let result = platform.get_view_task_template_profile(profile_id).await?;
            let output = if as_prompts {
                let registry = PreparedChoiceRegistry::new();
                let cache = ChoiceRegistryCache::from(
                    &registry as &dyn ChoiceRegistry<_>);
                let uvpr: UserViewProfileRef = (&result, cache).into();
                conf.serde_kind.to_string(&uvpr)?
            } else {
                conf.serde_kind.to_string(&result)?
            };
            println!("{output}");
        },
        ProfileCmd::Assign { profile_id, vtt_id } => {
            ProfileViewsBackend::insert_profile_views(
                platform.mc_platform.as_ref(),
                profile_id,
                vtt_id,
            ).await?;
            println!("assigned ViewTaskTemplate {vtt_id} to Profile {profile_id}");
        },
        ProfileCmd::Remove { profile_id, vtt_id } => {
            ProfileViewsBackend::delete_profile_views(
                platform.mc_platform.as_ref(),
                profile_id,
                vtt_id,
            ).await?;
            println!("removed ViewTaskTemplate {vtt_id} from Profile {profile_id}");
        },
    }
    Ok(())
}

async fn parse_task<'p>(
    platform: &'p Platform,
    arg: TaskCmd,
) -> anyhow::Result<()> {
    match arg {
        TaskCmd::ExecOneShot => {
            match platform.start_task().await? {
                Some(tec) => {
                    match tec.execute().await? {
                        (0, true) => println!("OK: task execution successful; exposure file view set"),
                        (0, false) => println!("ERR: task execution successful: exposure file view NOT set"),
                        _ => println!("ERR: task execution FAILED"),
                    }
                }
                None => {
                    println!("no outstanding job");
                }
            };
        }
    }
    Ok(())
}

async fn parse_vtt<'p>(
    platform: &'p Platform,
    arg: VttCmd,
) -> anyhow::Result<()> {
    let conf = CONF.get().expect("config is set by main");
    match arg {
        VttCmd::Import { input } => {
            let id = platform.adds_view_task_template(
                match input {
                    Some(path) => conf.serde_kind.from_reader(BufReader::new(fs::File::open(path)?))?,
                    None => conf.serde_kind.from_reader(BufReader::new(stdin()))?,
                }
            ).await?;
            println!("imported ViewTaskTemplate {id}");
        }
        VttCmd::Link { task_template_id, view_key, description } => {
            let id = ViewTaskTemplateBackend::insert_view_task_template(
                platform.mc_platform.as_ref(),
                &view_key,
                &description,
                task_template_id,
            ).await?;
            println!(
                "created ViewTaskTemplate {id} linked to TaskTemplate {task_template_id} with \
                view_key {view_key}"
            );
        }
        VttCmd::Export { id } => {
            let result = platform.get_view_task_template(id).await?;
            let output = conf.serde_kind.to_string(&result)?;
            println!("{output}");
        }
    }
    Ok(())
}

async fn parse_file_profile<'p>(
    platform: &'p Platform,
    exposure_file_id: i64,
    arg: FileProfileCmd,
) -> anyhow::Result<()> {
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

async fn parse_exposure_path<'p>(
    platform: &'p Platform,
    exposure_id: i64,
    path: &str,
    arg: ExposurePathCmd,
) -> anyhow::Result<()> {
    let ec = platform.get_exposure(exposure_id).await?;
    let efc = ec.ctrl_path(path).await?;
    let conf = CONF.get().expect("config is set by main");
    match arg {
        ExposurePathCmd::Assign { profile_id } => {
            let exposure_file_id = efc.exposure_file().id();
            let vttp = platform.get_view_task_template_profile(profile_id).await?;
            efc.set_vttprofile(vttp).await?;
            println!("profile set: exposure_file_id {exposure_file_id} => profile_id {profile_id}");
        },
        ExposurePathCmd::Prompts => {
            let efvttsc = efc.build_vttc().await?;
            let upgr = efvttsc.create_user_prompt_groups()?;
            let output = conf.serde_kind.to_string(&upgr)?;
            println!("{output}");
        },
        ExposurePathCmd::Answer { arg_id, answer } => {
            let efvttsc = efc.build_vttc().await?;

            // store the answer anyway.
            let user_input = UserInputMap::from([
                (arg_id, answer.clone()),
            ]);
            let id = efc.exposure_file().id();
            ExposureFileProfileBackend::update_ef_user_input(
                platform.mc_platform.as_ref(),
                id,
                &user_input,
            ).await?;

            // validation
            let arg = efvttsc.get_arg(&arg_id)
                .expect("provided arg_id is not part of the selected profile");
            match TaskArgBuilder::try_from((
                Some(answer.as_ref()),
                arg,
                efvttsc.get_registry_cache()?,
            )) {
                Err(error) => println!("Error: {error}"),
                Ok(_) => println!("OK"),
            }
        },
        ExposurePathCmd::Answers => {
            let id = efc.exposure_file().id();

            let efvttsc = efc.build_vttc().await?;
            let uargs = UserArgs::from(&efvttsc.create_user_arg_refs()?);

            if let Some(profile) = ExposureFileProfileBackend::get_ef_profile(
                platform.mc_platform.as_ref(),
                id,
            ).await? {
                let cache = efvttsc.get_registry_cache()?;

                for user_arg in uargs.iter() {
                    let arg = efvttsc.get_arg(&user_arg.id)
                        .expect("the arg that formed this user_arg be present");
                    let prompt = &user_arg.prompt;
                    let answer = profile.user_input.get(&user_arg.id);
                    let builder = TaskArgBuilder::try_from((
                        answer.map(|s| s.as_ref()),
                        arg,
                        cache.clone(),
                    ));

                    println!("Q: {prompt}");
                    println!("A: {answer:?}");

                    if let Some(error) = builder.map_err(|e| e.to_string()).err() {
                        println!("Error: {error}");
                    }

                    println!("");
                }
            } else {
                let path = efc.exposure_file().workspace_file_path();
                println!("no profile assigned for exposure {exposure_id} file {path}");
            }
        },
        ExposurePathCmd::Task { submit } => {
            let id = efc.exposure_file().id();
            let efvttsc = efc.build_vttc().await?;
            if let Some(profile) = ExposureFileProfileBackend::get_ef_profile(
                platform.mc_platform.as_ref(),
                id,
            ).await? {
                let vttc_tasks = efvttsc.create_tasks_from_input(
                    &profile.user_input
                )?;
                if submit {
                    let len = vttc_tasks.len();
                    efc.process_vttc_tasks(vttc_tasks).await?;
                    println!("{len} task(s) queued");
                } else {
                    let output = conf.serde_kind.to_string(&vttc_tasks)?;
                    println!("The generated VTTCTasks:");
                    println!("{output}");
                };
            } else {
                let path = efc.exposure_file().workspace_file_path();
                println!("no profile assigned for exposure {exposure_id} file {path}");
            }
        },
        ExposurePathCmd::Views => {
            let ef = efc.exposure_file();
            let views = ef.views().await?;
            println!("The following exposure file views are available:");
            for view in views.iter() {
                match view.view_key() {
                    Some(view_key) => println!("- {view_key}"),
                    _ => (),
                }
            }
        },
        // TODO need a way to pull up the latest VTTCTask?
        // figure out how to queue tasks without causing too much conflict
        // with new tasks that come up
    }

    Ok(())
}
