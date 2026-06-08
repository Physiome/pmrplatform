use std::{
    fs::File,
    io::{BufReader, stdin},
    path::PathBuf,
};
use clap::{Parser, Subcommand, ValueEnum};
use pmrcore::{
    ac::workflow::state::State,
    platform::ConnectorOption,
    workspace::traits::WorkspaceBackend,
};
use pmrdb::Backend;
use pmrctrl::platform::Builder;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
struct Entry {
    alias: String,
    description: Option<String>,
    long_description: Option<String>,
    path: String,
    url: String,
    workflow_state: State,

    creation_date: i64,
    effective_date: Option<i64>,
}

#[derive(Debug, Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    #[clap(flatten)]
    platform_builder: Builder,
    #[clap(short = 'v', long = "verbose", action = clap::ArgAction::Count)]
    verbose: u8,
}

#[derive(ValueEnum, Clone, Debug)]
enum ImportMode {
    Copy,
    Rename,
    Symlink,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Import {
        mode: ImportMode,
        origin_root: PathBuf,
        input: Option<PathBuf>, // the json file
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    let args = Cli::parse();
    stderrlog::new()
        .module(module_path!())
        .module("pmrctrl")
        .module("pmrdb")
        .module("pmrtqs")
        .verbosity((args.verbose as usize) + 1)
        .timestamp(stderrlog::Timestamp::Second)
        .init()
        .unwrap();

    let platform = args.platform_builder.clone().build().await
        .map_err(anyhow::Error::from_boxed)?;

    let mc = Backend::mc(ConnectorOption::from(&args.platform_builder.pmrapp_db_url)).await
        .map_err(anyhow::Error::from_boxed)?;
    // FIXME: may need to feature gate by db type due to how the erased dynamic type cannot be raw,
    // and that it's not possible to have the `Backend` helper provide truly dynamic raw access as that
    // exposes a concrete type that kills the dynamic nature of this, also that the sqlx executor isn't
    // dyn compatible...
    // let mc_backend = mc.as_ref().backend();
    let pc = Backend::pc(ConnectorOption::from(&args.platform_builder.pmrpc_db_url)).await
        .map_err(anyhow::Error::from_boxed)?;

    match args.command {
        Commands::Import { input, origin_root, mode } => {
            let entries: Vec<Entry> = match input {
                Some(path) => serde_json::from_reader(BufReader::new(File::open(path)?))?,
                None => serde_json::from_reader(BufReader::new(stdin()))?,
            };

            // alias, url can be derived from path, but this will be assumed to
            // be set by the `pdbg_workspace_export.py` script to be executed
            // inside the Zope/Plone debug shell.
            for Entry { alias, description, long_description, path, url, workflow_state, .. } in entries.into_iter() {
                // create the workspace and alias entries, set the workflow state
                let workspace_id = WorkspaceBackend::add_workspace(
                    platform.mc_platform.as_ref(),
                    &url,
                    description.as_deref(),
                    long_description.as_deref(),
                ).await?;
                platform.mc_platform.add_alias(
                    "workspace",
                    workspace_id,
                    &alias,
                ).await?;
                platform.ac_platform.set_wf_state_for_res(
                    &format!("/workspace/{workspace_id}/"),
                    workflow_state,
                ).await?;

                // then find the target .git, and move/copy/symlink depending on what got specified
                // path is assumed to always start with `/`.
                let mut origin = origin_root.join(&path[1..]);
                origin.push(".git");
                match mode {
                    ImportMode::Copy => todo!(),
                    ImportMode::Rename => todo!(),
                    ImportMode::Symlink => {
                        #[cfg(not(target_family = "unix"))]
                        unimplemented!();
                        #[cfg(target_family = "unix")]
                        {
                            use std::os::unix::fs;
                            fs::symlink(origin, platform.repo_root().join(workspace_id.to_string()))?;
                        }
                    }
                }

            }
        },
    }

    Ok(())
}
