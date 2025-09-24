use std::{
    collections::HashMap,
    fs,
    io::{BufReader, stdin},
    path::PathBuf,
};
use clap::{Parser, Subcommand};
use pmrcore::{
    ac::workflow::state::State,
    exposure::traits::Exposure,
};
use pmrctrl::platform::{Builder, Platform};
use serde::{Deserialize, Serialize};

mod error{
    use std::{
        error::Error,
        fmt::{Display, Formatter, Result},
    };

    #[derive(Debug)]
    pub struct ErrorMsg(pub String);

    impl Display for ErrorMsg {
        fn fmt(&self, f: &mut Formatter) -> Result {
            self.0.fmt(f)
        }
    }

    impl Error for ErrorMsg {}
}

use error::ErrorMsg;

#[derive(Debug, Deserialize, Serialize)]
pub struct ExposureEntry {
    pub path: String,
    pub workflow_state: State,
    pub wizard_export: WizardExport,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WizardExport(Vec<EntryRow>);

#[derive(Debug, Deserialize, Serialize)]
pub struct EntryRow(String, Record);

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Record {
    // the top exposure record
    Top(Top),
    // definition per file
    File(File),
    // definition per file
    Folder(Folder),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Top {
    pub commit_id: String,
    pub title: Option<String>,
    pub workspace: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct File {
    pub views: Vec<View>,
    pub file_type: Option<String>,
    pub selected_view: Option<String>,
    pub docview_gensource: Option<String>,
    pub docview_generator: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Folder {
    pub docview_gensource: Option<String>,
    pub docview_generator: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct View(pub String, pub Option<HashMap<String, Option<Value>>>);

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Value {
    Str(String),
    Bool(bool),
    Flags(HashMap<String, Vec<String>>),
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

#[derive(Debug, Subcommand)]
enum Commands {
    Import {
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
        Commands::Import { input } => process_exposure_entries(
            &platform,
            match input {
                Some(path) => serde_json::from_reader(BufReader::new(fs::File::open(path)?))?,
                None => serde_json::from_reader(BufReader::new(stdin()))?,
            },
        ).await?,
    }

    Ok(())
}

async fn process_exposure_entries(
    platform: &Platform,
    exposure_entries: Vec<ExposureEntry>,
) -> anyhow::Result<()> {
    for ExposureEntry {
        path,
        workflow_state,
        wizard_export,
    } in exposure_entries.into_iter() {
        process_wizard_export(
            platform,
            path,
            wizard_export,
            workflow_state,
        ).await?
    }
    Ok(())
}

async fn process_wizard_export(
    platform: &Platform,
    exposure_path: String,
    wizard_export: WizardExport,
    workflow_state: State,
) -> anyhow::Result<()> {
    let mut top = None::<Top>;
    let mut files = Vec::<(String, File)>::new();

    for EntryRow(target, record) in wizard_export.0.into_iter() {
        match record {
            Record::Top(t) => {
                top.replace(t);
            }
            Record::File(ef) => {
                files.push((target, ef));
            }
            Record::Folder(_) => (),
        }
    }
    // with the top, create exposure
    // with the files, create exposure files in the exposure
    let Top { commit_id, workspace, ..  } = top.ok_or(ErrorMsg(
        format!("no top level item found for {exposure_path}")
    ))?;
    println!("{exposure_path} file count: {}", files.len());

    let alias = workspace
        .replace("/pmr/", "")
        // this should maintain the id for default case
        .replace("workspace/", "")
        // drop the prefix for users and replace sep with `-`
        .replace("w/", "")
        .replace("/", "-");

    let workspace_id = platform.mc_platform
        .resolve_alias("workspace", &alias)
        .await?
        .ok_or(ErrorMsg(format!("workspace under alias {alias} not found ({workspace:?} not imported?)")))?;

    let ec = platform.create_exposure(
        workspace_id,
        &commit_id,
    ).await?;
    let exposure_id = ec.exposure().id();
    platform.ac_platform.set_wf_state_for_res(
        &format!("/exposure/{exposure_id}/"),
        workflow_state,
    ).await?;

    for (path, file) in files.into_iter() {
        let efc = ec.create_file(&path).await?;
        // let vtt_profile = platform.get_view_task_template_profile(profile_id).await?;
        // efc.set_vttprofile(vtt_profile).await?
    }


    // TODO create the files and views
    // TODO alias for the exposure itself

    Ok(())
}
