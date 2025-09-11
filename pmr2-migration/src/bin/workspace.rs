use std::{
    fs::File,
    io::{BufReader, stdin},
    path::PathBuf,
};
use clap::{Parser, Subcommand, ValueEnum};
use pmrcore::{
    ac::workflow::state::State,
    workspace::traits::WorkspaceBackend,
};
use pmrctrl::platform::Builder;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
struct Entry {
    alias: String,
    description: Option<String>,
    path: String,
    url: String,
    workflow_state: State,
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

    let platform = args.platform_builder.build().await
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
            for Entry { alias, description, path, url, workflow_state } in entries.into_iter() {
                // create the workspace and alias entries, set the workflow state
                let workspace_id = WorkspaceBackend::add_workspace(
                    platform.mc_platform.as_ref(),
                    &url,
                    description.as_deref(),
                    None,
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
