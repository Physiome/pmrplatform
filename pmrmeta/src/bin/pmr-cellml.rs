use clap::{Parser, Subcommand};
use pmrmeta::{
    cellml::query,
    read::xml_to_store,
};
use pmrctrl::platform::Builder as PlatformBuilder;
use std::{
    fs,
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
    Index {
        #[clap(long)]
        input_path: String,
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
        Commands::Index { input_path, exposure_id, exposure_path } => {
            let reader = BufReader::new(fs::File::open(input_path)?);
            let resource_path = format!("/exposure/{exposure_id}/{exposure_path}");
            let store = xml_to_store(reader)?;

            // Store the citation for the incoming file
            let citations = query::pubmed_id(&store)?;
            for citation in citations.iter() {
                platform.pc_platform.add_citation(&citation).await.ok();
                platform.pc_platform.link_citation(&citation, &resource_path).await.ok();
            }

            // Add the various information acquired from the metadata into the index
            let keywords = query::keywords(&store)?;
            platform.pc_platform.resource_link_kind_with_terms(
                "cellml_keyword",
                &resource_path,
                &mut keywords.iter()
                    .map(String::as_ref),
            ).await?;

        }
    }

    Ok(())
}
