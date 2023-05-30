use sqlx::{
    Sqlite,
    migrate::MigrateDatabase,
};
use std::env;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process;
use structopt::StructOpt;

use pmrmodel::backend::db::{
    Profile,
    SqliteBackend,
};
use pmrmodel::model::workspace::{
    WorkspaceBackend,
    stream_workspace_records_default,
    stream_workspace_records_as_json,
};
use pmrmodel::model::workspace_alias::WorkspaceAliasBackend;
use pmrmodel::model::workspace_sync::WorkspaceSyncBackend;
use pmrmodel::model::workspace_tag::WorkspaceTagBackend;

use pmrrepo::git::{
    PmrBackendW,
    PmrBackendWR,

    stream_git_result_as_json,
    stream_git_result_default,
};

#[derive(StructOpt)]
struct Args {
    #[structopt(subcommand)]
    cmd: Option<Command>,

    #[structopt(short = "v", long = "verbose", parse(from_occurrences))]
    verbose: usize,

    #[structopt(short = "j", long = "json")]
    json: bool,
}

#[derive(StructOpt)]
enum Command {
    Register {
        url: String,
        description: String,
        #[structopt(short = "l", long = "longdesc", default_value = "")]
        long_description: String,
    },
    Update {
        workspace_id: i64,
        description: String,
        #[structopt(short = "l", long = "longdesc", default_value = "")]
        long_description: String,
    },
    Sync {
        workspace_id: i64,
        #[structopt(short, long)]
        log: bool,
    },
    Tags {
        workspace_id: i64,
        #[structopt(short, long)]
        index: bool,
    },
    Blob {
        workspace_id: i64,
        #[structopt(short, long)]
        obj_id: String,
    },
    Info {
        workspace_id: i64,
        #[structopt(short, long)]
        commit_id: Option<String>,
        #[structopt(short, long)]
        path: Option<String>,
        #[structopt(short, long)]
        raw: bool,
    },
    Log {
        workspace_id: i64,
        #[structopt(short, long)]
        commit_id: Option<String>,
    },
    Alias {
        workspace_id: i64,
        #[structopt(short = "a", long = "alias")]
        alias: Option<String>,
        // TODO include reverse lookup here?
    },
}

fn fetch_envvar(key: &str) -> anyhow::Result<String> {
    match env::var(&key) {
        Err(e) => {
            writeln!(&mut io::stderr(), "couldn't interpret {}: {}", key, e)?;
            process::exit(1);
        },
        Ok(val) => Ok(val),
    }
}

#[async_std::main]
#[paw::main]
async fn main(args: Args) -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    stderrlog::new()
        .module(module_path!())
        .verbosity(args.verbose + 1)
        .timestamp(stderrlog::Timestamp::Second)
        .init()
        .unwrap();

    // TODO make this be sourced from a configuration file of sort...
    let git_root = PathBuf::from(fetch_envvar("PMR_GIT_ROOT")?);
    let db_url = fetch_envvar("DATABASE_URL")?;
    if !Sqlite::database_exists(&db_url).await.unwrap_or(false) {
        log::warn!("database {} does not exist; creating...", &db_url);
        Sqlite::create_database(&db_url).await?
    }
    let backend = SqliteBackend::from_url(&db_url)
        .await?
        .run_migration_profile(Profile::Pmrapp)
        .await?;

    match args.cmd {
        Some(Command::Register { url, description, long_description }) => {
            println!("Registering workspace with url '{}'...", &url);
            let workspace_id = WorkspaceBackend::add_workspace(&backend, &url, &description, &long_description).await?;
            println!("Registered workspace with id {}", workspace_id);
        }
        Some(Command::Update { workspace_id, description, long_description }) => {
            println!("Updating workspace with id {}...", workspace_id);
            if WorkspaceBackend::update_workspace(&backend, workspace_id, &description, &long_description).await? {
                println!("Updated workspace id {}", workspace_id);
            }
            else {
                println!("Invalid workspace id {}", workspace_id);
            }
        }
        Some(Command::Sync { workspace_id, log }) => {
            if log {
                println!("Listing of sync logs for workspace with id {}", workspace_id);
                let recs = WorkspaceSyncBackend::get_workspaces_sync_records(&backend, workspace_id).await?;
                println!("start - end - status");
                for rec in recs {
                    println!("{}", rec);
                }
            }
            else {
                println!("Syncing commits for workspace with id {}...", workspace_id);
                let workspace = WorkspaceBackend::get_workspace_by_id(&backend, workspace_id).await?;
                let pmrbackend = PmrBackendW::new(&backend, git_root, &workspace);
                pmrbackend.git_sync_workspace().await?;
            }
        }
        Some(Command::Tags { workspace_id, index }) => {
            if index {
                println!("Indexing tags for workspace with id {}...", workspace_id);
                let workspace = WorkspaceBackend::get_workspace_by_id(&backend, workspace_id).await?;
                let pmrbackend = PmrBackendWR::new(&backend, git_root, &workspace)?;
                pmrbackend.index_tags().await?;
            }
            else {
                println!("Listing of indexed tags workspace with id {}", workspace_id);
                let recs = WorkspaceTagBackend::get_workspace_tags(&backend, workspace_id).await?;
                println!("commit_id - tag");
                for rec in recs {
                    println!("{}", rec);
                }
            }
        }
        Some(Command::Blob { workspace_id, obj_id }) => {
            let workspace = WorkspaceBackend::get_workspace_by_id(&backend, workspace_id).await?;
            let pmrbackend = PmrBackendWR::new(&backend, git_root, &workspace)?;
            pmrbackend.get_obj_by_spec(&obj_id).await?;
        }
        Some(Command::Info { workspace_id, commit_id, path, raw }) => {
            let workspace = WorkspaceBackend::get_workspace_by_id(&backend, workspace_id).await?;
            let pmrbackend = PmrBackendWR::new(&backend, git_root, &workspace)?;
            if raw {
                let git_result = pmrbackend.pathinfo(
                    commit_id.as_deref(), path.as_deref())?;
                pmrbackend.stream_result_blob(
                    io::stdout(), &git_result).await?;
            }
            else {
                if args.json {
                    let git_result = pmrbackend.pathinfo(
                        commit_id.as_deref(), path.as_deref())?;
                    stream_git_result_as_json(
                        io::stdout(), &git_result)?;
                }
                else {
                    let git_result = pmrbackend.pathinfo(
                        commit_id.as_deref(), path.as_deref())?;
                    stream_git_result_default(
                        io::stdout(), &git_result)?;
                }
            }
        }
        Some(Command::Log { workspace_id, commit_id }) => {
            let workspace = WorkspaceBackend::get_workspace_by_id(&backend, workspace_id).await?;
            let pmrbackend = PmrBackendWR::new(&backend, git_root, &workspace)?;
            let logs = pmrbackend.loginfo(commit_id.as_deref(), None)?;
            if args.json {
                // stream_git_result_as_json(io::stdout(), &logs)?;
                let writer = io::stdout();
                serde_json::to_writer(writer, &logs)?;
            }
            else {
                // stream_git_result_default(io::stdout(), &logs)?;
                let mut writer = io::stdout();
                writer.write(format!("have log_info {:?}", logs).as_bytes())?;
            }
        }
        Some(Command::Alias { workspace_id, alias }) => {
            if alias.is_none() {
                let aliases = WorkspaceAliasBackend::get_aliases(&backend, workspace_id).await?;
                println!("Printing list of all aliases");
                for rec in aliases {
                    println!("{}", rec);
                }
            }
            else {
                let alias = alias.unwrap();
                WorkspaceAliasBackend::add_alias(&backend, workspace_id, &alias).await?;
                println!("setting alias to {}", alias);
            }
        }
        None => {
            let recs = WorkspaceBackend::list_workspaces(&backend).await?;
            if args.json {
                stream_workspace_records_as_json(io::stdout(), recs)?;
            }
            else {
                println!("Printing list of all workspaces");
                stream_workspace_records_default(io::stdout(), recs)?;
            }
        }
    }

    Ok(())
}
