use std::{
    collections::HashMap,
    fs,
    io::{BufReader, stdin},
    path::PathBuf,
    sync::{LazyLock, OnceLock},
};
use clap::{Parser, Subcommand};
use pmrcore::{
    ac::workflow::state::State,
    exposure::traits::{Exposure, ExposureFile},
    task_template::UserInputMap,
};
use pmrctrl::platform::{Builder, Platform};
use serde::{Deserialize, Serialize};

mod error {
    use std::error::Error;
    use super::ErrorMsg;

    impl Error for ErrorMsg {}
}

mod fmt {
    use super::{ErrorMsg, Value};
    use std::fmt::{Display, Formatter, Result};

    impl Display for ErrorMsg {
        fn fmt(&self, f: &mut Formatter) -> Result {
            self.0.fmt(f)
        }
    }

    impl Display for Value {
        fn fmt(&self, f: &mut Formatter) -> Result {
            match self {
                Self::Str(s) => s.fmt(f),
                Self::Bool(b) => b.fmt(f),
                // there are currently no new types that use the mapping flags
                Self::Flags(_) => unreachable!(),
            }
        }
    }
}

#[derive(Debug)]
pub struct ErrorMsg(pub String);

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

/// For looking up the corresponding value from a `UserArg` in the `View` struct
#[derive(Debug)]
pub struct ViewsMap(HashMap<String, Option<HashMap<String, Option<Value>>>>);

impl From<Vec<View>> for ViewsMap {
    fn from(views: Vec<View>) -> Self {
        Self(views.into_iter()
            .map(|view| (view.0, view.1))
            .collect())
    }
}

impl ViewsMap {
    const PROMPT_TO_NAME: LazyLock<HashMap<&str, (&str, &str)>> = LazyLock::new(|| {
        [
            ("Documentation source", ("docgen", "source")),
            ("Documentation type", ("docgen", "generator")),
            ("Citation file format", ("licence_citation", "format")),
            ("License", ("licence_citation", "dcterms_license")),
        ]
        .into_iter()
        .collect()
    });

    const PROMPT_ANSWER_REMAP: LazyLock<HashMap<&str, HashMap<Option<&str>, &str>>> = LazyLock::new(|| {
        [
            ("Documentation type", vec![
                (Some("safe_html"), "HTML document"),
                (Some("cellml_tmpdoc"), "Legacy CellML-tmpdoc"),
                (Some("rest_to_html"), "ReStructuredText"),
            ]),
            ("Citation file format", vec![
                (None, "(None; manually pick license)"),
                (Some("cellml_rdf_metadata"), "CellML RDF Metadata"),
            ]),
            ("License", vec![
                // defualt license to CC 3.0 Unported
                (None, "Creative Commons - Attributions 3.0 Unported"),
                (Some("http://creativecommons.org/licenses/by/3.0/"), "Creative Commons - Attributions 3.0 Unported"),
            ]),
        ]
        .into_iter()
        .map(|(key, value)| (key, value.into_iter().collect()))
        .collect()
    });

    /// The intent of this function is to allow iteration down some user arg paring,
    /// such that the incoming prompt will have the value resolved.
    pub fn lookup(&self, prompt: &str) -> Option<String> {
        if let Some((view_key, field_name)) = Self::PROMPT_TO_NAME.get(prompt) {
            if let Some(map) = self.0.get(*view_key).map(Option::as_ref).flatten() {
                map.get(*field_name).map(Option::as_ref).flatten().map(Value::to_string)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn remap_answer(&self, prompt: &str, answer: Option<String>) -> Option<String> {
        if let Some(answer_table) = Self::PROMPT_ANSWER_REMAP.get(prompt) {
            answer_table.get(&answer.as_deref())
                .as_ref()
                .map(|s| Some(s.to_string()))
                .unwrap_or(answer)
        } else {
            answer
        }
    }
}

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

struct Site {
    legacy_map: HashMap<&'static str, &'static str>,
    profile_map: HashMap<String, i64>,
    default_profile_id: i64,
}

impl Site {
    fn file_type_to_profile_id(&self, file_type: Option<&str>) -> i64 {
        if let Some(file_type) = file_type {
            *self.legacy_map
                .get(file_type)
                .map(|profile_name| self.profile_map.get(*profile_name))
                .flatten()
                .unwrap_or(&self.default_profile_id)
        } else {
            self.default_profile_id
        }
    }
}

static SITE: OnceLock<Site> = OnceLock::new();

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

    prepare_defaults(&platform).await?;

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

async fn prepare_defaults(platform: &Platform) -> anyhow::Result<()> {
    let legacy_map = [
        ("/pmr/filetype/cellml", "CellML Model"),
        ("/pmr/filetype/argon_sds_archive", "Argon SDS Archive"),
        ("/pmr/filetype/other", "Documentation Only"),
    ]
    .into_iter()
    .collect::<HashMap<_, _>>();

    let profile_map = platform.mc_platform
        .list_profiles()
        .await?
        .into_iter()
        .map(|p| (p.title, p.id))
        .collect::<HashMap<_, _>>();

    let default_profile_id = *profile_map.get("Documentation Only")
        .expect("'Documentation Only' profile must be available for default");

    eprintln!("'Documentation Only' profile available as profile id {default_profile_id}");
    SITE.set(Site {
        legacy_map,
        profile_map,
        default_profile_id,
    })
    .map_err(|_| ())
    .expect("default shouldn't be already set");
    Ok(())
}

async fn site() -> &'static Site {
    SITE.get().expect("site should have been provided")
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

    let site = site().await;

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
    eprintln!("Processing: {exposure_path}; file count: {}", files.len());

    let workspace_alias = workspace
        .replace("/pmr/", "")
        // this should maintain the id for default case
        .replace("workspace/", "")
        // drop the prefix for users and replace sep with `-`
        .replace("w/", "")
        .replace("/", "-");

    let workspace_id = platform.mc_platform
        .resolve_alias("workspace", &workspace_alias)
        .await?
        .ok_or(ErrorMsg(format!(
            "workspace under alias {workspace_alias} not found ({workspace:?} not imported?)"
        )))?;

    let alias = exposure_path
        .replace("/pmr/", "")
        // this effectively unifies the original long ids
        .replace("exposure/", "")
        .replace("e/", "")
        // deal with random paths as is; though this case shouldn't exist in production
        .replace("/", "-");

    let ec = match platform.create_exposure(
        workspace_id,
        &commit_id,
    ).await {
        Ok(ec) => ec,
        Err(_) => {
            eprintln!("The commit {commit_id} might be missing, resyncing from upstream");
            platform.repo_backend()
                .sync_workspace(workspace_id).await?;
            platform.create_exposure(workspace_id, &commit_id).await?
        }
    };
    platform.mc_platform.add_alias(
        "exposure",
        ec.exposure().id(),
        &alias,
    // ).await?;
    ).await.ok();
    let exposure_id = ec.exposure().id();
    platform.ac_platform.set_wf_state_for_res(
        &format!("/exposure/{exposure_id}/"),
        workflow_state,
    ).await?;

    // needed to deal with lifetime issues associated with the `EFViewTemplatesCtrl`
    let mut cache = Vec::new();

    for (path, file) in files.into_iter() {
        let efc = ec.create_file(&path).await?;
        let profile_id = site.file_type_to_profile_id(file.file_type.as_deref());
        let vtt_profile = platform.get_view_task_template_profile(profile_id).await?;
        efc.set_vttprofile(vtt_profile).await?;
        let id = efc.exposure_file().id();
        let efvttsc = efc.try_into_vttc().await?;
        let user_args = efvttsc.create_user_args()?
            .into_iter()
            .map(|uar| (uar.id, uar.prompt))
            .collect::<Vec<_>>();

        let user_input = map_view_to_user_args(file.views.into(), user_args)
            .into_iter()
            .filter_map(|(id, prompt, answer)| {
                if let Some(answer) = answer {
                    Some((id, answer))
                } else {
                    eprintln!("{prompt} failed to be answered");
                    None
                }
            })
            .collect::<UserInputMap>();

        // store the inputs for now
        platform.mc_platform.update_ef_user_input(id, &user_input).await?;

        // lifetime issues here
        // let vttc_tasks = efvttsc.create_tasks_from_input(&user_input)?;
        // efvttsc.exposure_file_ctrl()
        //     .process_vttc_tasks(vttc_tasks).await?;

        cache.push((efvttsc, user_input));
    }

    for (efvttsc, user_input) in cache.iter() {
        match efvttsc.create_tasks_from_input(user_input) {
            Ok(vttc_tasks) => {
                let tasks = efvttsc.exposure_file_ctrl().process_vttc_tasks(vttc_tasks).await?;
                eprintln!("Successfully created {} tasks", tasks.len());
            }
            Err(e) => eprintln!("{e}"),
        }
    }

    Ok(())
}

fn map_view_to_user_args(views: ViewsMap, user_args: Vec<(i64, String)>) -> Vec<(i64, String, Option<String>)> {
    // dbg!(&views);
    // dbg!(&user_args);
    user_args.into_iter()
        .map(|(id, prompt)| {
            let extracted = views.lookup(&prompt);
            let result = views.remap_answer(&prompt, extracted);
            (id, prompt, result)
        })
        .collect()
}
