use clap::{Parser, Subcommand};
use pmrmeta::cellml::cmeta::{Cmeta, Pmr2Cmeta};
use pmrcore::exposure::traits::Exposure as _;
use pmrctrl::platform::Builder as PlatformBuilder;
use std::{
    fs,
    path::Path,
    io::BufReader,
    sync::OnceLock,
};

#[derive(Debug, Parser)]
struct Config {
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
    Cmeta {
        #[clap(long)]
        input_path: String,
        #[clap(long)]
        output_dir: String,
        #[clap(long)]
        exposure_id: i64,
        #[clap(long)]
        exposure_path: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    let args = Cli::parse();
    stderrlog::new()
        .module(module_path!())
        .module("pmrdb")
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
        Commands::Cmeta { input_path, output_dir, exposure_id, exposure_path } => {
            let resource_path = format!("/exposure/{exposure_id}/{exposure_path}");

            let reader = BufReader::new(fs::File::open(input_path)?);
            let cmeta = Cmeta::new(reader)?;

            let cmeta_id = cmeta.root_cmetaid();
            let title = cmeta.dc_title(Some(""))?;
            let citations = cmeta.citation(cmeta_id.map(|s| format!("#{s}")).as_deref())?;
            let vcard = cmeta.dc_vcard_info(Some(""))?;
            let keywords = cmeta.contextual_keywords()?;

            // first gather the data for the output file
            let mut pmr2_cmeta = Pmr2Cmeta::default();
            pmr2_cmeta.keywords = Some(keywords.clone());
            pmr2_cmeta.model_title = title.get(0).take().cloned();
            if let Some(vcard) = vcard.get(0) {
                pmr2_cmeta.model_author = vcard.fullname();
                pmr2_cmeta.model_author_org = vcard.org();
            }
            if let Some(citations) = citations.get(0) {
                pmr2_cmeta.citation_bibliographic_citation = citations.journal.clone();
                pmr2_cmeta.citation_authors = Some(citations.authors
                    .iter()
                    .map(|a| (
                        a.family.to_string(),
                        a.given.clone().unwrap_or_default(),
                        a.other.clone().unwrap_or_default(),
                    ))
                    .collect()
                );
                pmr2_cmeta.citation_title = Some(citations.title.clone());
                pmr2_cmeta.citation_id = Some(citations.id.clone());
                pmr2_cmeta.citation_issued = citations.issued.clone();
            }
            pmr2_cmeta.citations = citations;

            // write the output to the file
            let output_dir = fs::File::create(Path::new(&output_dir).join("cmeta.json"))?;
            serde_json::to_writer(output_dir, &pmr2_cmeta)?;

            // with the data gathered, populate the index
            // Only index the first alias created, or the id if that's not found.
            let alias = platform
                .mc_platform
                .get_alias("exposure", exposure_id)
                .await?
                .unwrap_or(exposure_id.to_string());
            platform.pc_platform.resource_link_kind_with_term(
                &resource_path,
                "exposure_alias",
                &alias,
            )
            .await?;
            platform.pc_platform.resource_link_kind_with_term(
                &resource_path,
                "aliased_uri",
                &format!("/exposure/{alias}/{exposure_path}")
            )
            .await?;
            // Add the various information acquired from the metadata into the index
            platform.pc_platform.resource_link_kind_with_terms(
                &resource_path,
                "cellml_keyword",
                &mut keywords.iter()
                    .map(|(_, kw)| kw.as_str()),
            )
            .await?;
            // aka title under PMR2
            platform.pc_platform.resource_link_kind_with_term(
                &resource_path,
                "description",
                // fallback from main title to citation title to generated one.
                &pmr2_cmeta.model_title.unwrap_or(
                    pmr2_cmeta.citation_title.unwrap_or(
                        format!("Exposure {exposure_id}")
                    )
                )
            )
            .await?;
            // citation

            for citation in pmr2_cmeta.citations.iter() {
                platform.pc_platform.add_citation(&citation).await.ok();
            }
            // Citation id.
            platform.pc_platform.resource_link_kind_with_terms(
                &resource_path,
                "citation_id",
                &mut pmr2_cmeta.citations
                    .iter()
                    .map(|citation| citation.id.as_ref()),
            )
            .await?;

            let exposure = platform.get_exposure(exposure_id).await?;
            platform.pc_platform.resource_link_kind_with_term(
                &resource_path,
                "created_ts",
                &exposure.exposure().created_ts().to_string(),
            )
            .await?;

            let file = exposure.ctrl_path(exposure_path).await?;
            let pathinfo = file.pathinfo();
            let repo = pathinfo.repo();
            if let Some(commit) = pathinfo.commit(&repo) {
                let seconds = commit.decode()?.author()?.time()?.seconds.to_string();
                platform.pc_platform.resource_link_kind_with_term(
                    &resource_path,
                    "commit_authored_ts",
                    &seconds.to_string(),
                )
                .await?;
            }

        }
    }

    Ok(())
}
