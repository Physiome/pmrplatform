use clap::{Parser, Subcommand};
use pmrmeta::citation::index;
use pmrctrl::platform::{
    Builder as PlatformBuilder,
    Platform,
};
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
    Add {
        identifier: String,
    },
    List {
        identifier: Option<String>,
    },
    #[command(arg_required_else_help = true)]
    Rdfxml {
        #[command(subcommand)]
        cmd: RdfxmlCmd,
    },
}

#[derive(Debug, Subcommand)]
enum RdfxmlCmd {
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
        Commands::Add { identifier } => {
            platform.pc_platform.add_citation(&identifier).await?;
        },
        Commands::List { identifier } => {
            if let Some(identifier) = identifier {

                match platform.pc_platform.list_resources(
                    "citation_id",
                    &identifier,
                ).await? {
                    Some(results) => {
                        println!("Listing of resources associated with citation {identifier}");
                        for resource_path in results.resource_paths.iter() {
                            println!("- {resource_path:?}");
                        }
                    }
                    None => {
                        println!("citation {identifier} not indexed?");
                    }
                }
            } else {
                let citations = platform.pc_platform.list_citations().await?;
                println!("Listing of citation identifiers recorded");
                for citation in citations.into_iter() {
                    println!("{}", citation.identifier);
                }
            }
        },
        Commands::Rdfxml { cmd } => {
            parse_rdfxml_cmd(&platform, cmd).await?;
        },
    }

    Ok(())
}

async fn parse_rdfxml_cmd<'p>(
    platform: &'p Platform,
    arg: RdfxmlCmd,
) -> anyhow::Result<()> {
    match arg {
        RdfxmlCmd::Index { input_path, exposure_id, exposure_path } => {
            let reader = BufReader::new(fs::File::open(input_path)?);
            let resource_path = format!("/exposure/{exposure_id}/{exposure_path}");
            let citations = index(reader)?;
            for citation in citations.iter() {
                platform.pc_platform.add_citation(&citation).await.ok();
                platform.pc_platform.resource_link_kind_with_term(
                    &resource_path,
                    "citation_id",
                    &citation,
                )
                .await?;
            }
        }
    }
    Ok(())
}
