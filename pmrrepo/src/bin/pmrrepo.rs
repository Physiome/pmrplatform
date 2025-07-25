use sqlx::{
    Sqlite,
    migrate::MigrateDatabase,
};
use std::env;
use std::io::{self, Write};
use std::ops::Deref;
use std::path::PathBuf;
use std::process;
use std::sync::Arc;
use structopt::StructOpt;

use pmrmodel::backend::db::{
    MigrationProfile,
    SqliteBackend,
};
use pmrmodel::model::workspace::{
    stream_workspace_records_default,
    stream_workspace_records_as_json,
};
use pmrcore::{
    repo::PathObjectInfo,
    workspace::traits::{
        WorkspaceBackend,
        WorkspaceSyncBackend,
        WorkspaceTagBackend,
    },
};
use pmrrepo::{
    backend::Backend,
    handle::GitHandleResult,
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
}

fn stream_git_result_default<'a>(
    mut writer: impl Write,
    item: &GitHandleResult<'a>,
    repo: &'a gix::Repository,
) -> std::result::Result<usize, std::io::Error> {
    // the repo argument must be spawned off the result
    // this wasn't an issue with an earlier design but this is required
    // to workaround issues with lifetime, even if everything is discarded
    // before function ends.
    writer.write(format!("
        have repo at {:?}
        have commit {:?}
        have commit_object {:?}
        using repopath {:?}
        have git_object {:?}
        have path_object_info {:?}
        \n",
        item.repo().path(),
        item.commit(&repo).map(|c| c.id()),
        item.commit(&repo),
        item.path(),
        &item.target(),
        <Option<PathObjectInfo>>::from(item),
    ).as_bytes())
}

fn stream_git_result_as_json(
    writer: impl Write,
    item: &GitHandleResult,
) -> Result<(), serde_json::Error> {
    serde_json::to_writer(writer, &<Option<PathObjectInfo>>::from(item))
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
    dotenvy::dotenv().ok();
    stderrlog::new()
        .module(module_path!())
        .verbosity(args.verbose + 1)
        .timestamp(stderrlog::Timestamp::Second)
        .init()
        .unwrap();

    // TODO make this be sourced from a configuration file of sort...
    let git_root = PathBuf::from(fetch_envvar("PMR_REPO_ROOT")?);
    let db_url = fetch_envvar("PMRAPP_DB_URL")?;
    if !Sqlite::database_exists(&db_url).await.unwrap_or(false) {
        log::warn!("database {} does not exist; creating...", &db_url);
        Sqlite::create_database(&db_url).await?
    }
    let platform = SqliteBackend::from_url(&db_url)
        .await?
        .run_migration_profile(MigrationProfile::Pmrapp)
        .await?;

    let backend = Backend::new(Arc::new(platform), git_root);
    let platform = backend.platform();

    match args.cmd {
        Some(Command::Register { url, description, long_description }) => {
            println!("Registering workspace with url '{}'...", &url);
            let workspace_id = WorkspaceBackend::add_workspace(
                platform,
                &url,
                &description,
                &long_description,
            ).await?;
            println!("Registered workspace with id {}", workspace_id);
        }
        Some(Command::Update { workspace_id, description, long_description }) => {
            println!("Updating workspace with id {}...", workspace_id);
            if WorkspaceBackend::update_workspace(
                platform,
                workspace_id,
                &description,
                &long_description,
            ).await? {
                println!("Updated workspace id {}", workspace_id);
            }
            else {
                println!("Invalid workspace id {}", workspace_id);
            }
        }
        Some(Command::Sync { workspace_id, log }) => {
            if log {
                println!("Listing of sync logs for workspace with id {}", workspace_id);
                let recs = WorkspaceSyncBackend::get_workspaces_sync_records(platform, workspace_id).await?;
                println!("start - end - status");
                for rec in recs {
                    println!("{}", rec);
                }
            }
            else {
                println!("Syncing commits for workspace with id {}...", workspace_id);
                let _ = backend.sync_workspace(workspace_id).await?;
            }
        }
        Some(Command::Tags { workspace_id, index }) => {
            if index {
                println!("Indexing tags for workspace with id {}...", workspace_id);
                let handle = backend.git_handle(workspace_id).await?;
                handle.index_tags().await?;
            }
            else {
                println!("Listing of indexed tags workspace with id {}", workspace_id);
                let recs = WorkspaceTagBackend::get_workspace_tags(platform, workspace_id).await?;
                println!("commit_id - tag");
                for rec in recs {
                    println!("{}", rec);
                }
            }
        }
        Some(Command::Blob { workspace_id, obj_id }) => {
            let handle = backend.git_handle(workspace_id).await?;
            let repo = handle.repo()?;
            let obj = repo.rev_parse_single(obj_id.deref())?.object()?;
            log::info!("Found object {} {}", obj.kind, obj.id);
            // info!("{:?}", object_to_info(&obj));
        }
        Some(Command::Info { workspace_id, commit_id, path, raw }) => {
            let handle = backend.git_handle(workspace_id).await?;
            let git_result = handle.pathinfo(
                commit_id.as_deref(), path.as_deref())?;
            if raw {
                git_result.stream_blob(io::stdout()).await?;
            }
            else {
                if args.json {
                    stream_git_result_as_json(
                        io::stdout(), &git_result)?;
                }
                else {
                    let repo = git_result.repo();
                    stream_git_result_default(
                        io::stdout(), &git_result, &repo)?;
                }
            }
        }
        Some(Command::Log { workspace_id, commit_id }) => {
            let handle = backend.git_handle(workspace_id).await?;
            let logs = handle.loginfo(commit_id.as_deref(), None, None)?;
            if args.json {
                let writer = io::stdout();
                serde_json::to_writer(writer, &logs)?;
            }
            else {
                let mut writer = io::stdout();
                writer.write(format!("have log_info {:?}", logs).as_bytes())?;
            }
        }
        None => {
            let workspaces = WorkspaceBackend::list_workspaces(platform).await?;
            if args.json {
                stream_workspace_records_as_json(io::stdout(), &workspaces)?;
            }
            else {
                println!("Printing list of all workspaces");
                stream_workspace_records_default(io::stdout(), &workspaces)?;
            }
        }
    }

    Ok(())
}
