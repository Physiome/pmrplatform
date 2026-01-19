use clap::{Parser, Subcommand};
use pmrctrl::platform::{
    Builder as PlatformBuilder,
    Platform,
};
use std::sync::OnceLock;

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
        #[command(subcommand)]
        cmd: IndexCmd,
    },
    #[command(arg_required_else_help = true)]
    Link {
        #[command(subcommand)]
        cmd: LinkCmd,
    },
    #[command(arg_required_else_help = true)]
    Query {
        #[clap(flatten)]
        cmd: QueryCmd,
    },
}

#[derive(Debug, Subcommand)]
enum IndexCmd {
    List {
        kind: Option<String>,
    },
    // TODO Drop { .. }
}

#[derive(Debug, Subcommand)]
enum LinkCmd {
    #[command(arg_required_else_help = true)]
    Create {
        resource_path: String,
        #[clap(long)]
        kind: String,
        #[clap(long, value_delimiter = ',')]
        terms: Vec<String>,
    },
    #[command(arg_required_else_help = true)]
    List {
        resource_path: String,
    },
    #[command(arg_required_else_help = true)]
    Forget {
        resource_path: String,
        kind: Option<String>,
    },
}

#[derive(Debug, Parser)]
struct QueryCmd {
    kind: String,
    term: String,
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
        Commands::Index { cmd } => {
            parse_index_cmd(&platform, cmd).await?;
        },
        Commands::Link { cmd } => {
            parse_link_cmd(&platform, cmd).await?;
        },
        Commands::Query { cmd } => {
            parse_query_cmd(&platform, cmd).await?;
        },
    }

    Ok(())
}

async fn parse_index_cmd(
    platform: &Platform,
    arg: IndexCmd,
) -> anyhow::Result<()> {
    match arg {
        IndexCmd::List { kind } => {
            match kind {
                Some(kind) => {
                    match platform.pc_platform.list_terms(&kind).await? {
                        Some(terms) => {
                              println!(
                                  "Under index of kind {:?} are {} terms, they are:",
                                  terms.kind.description,
                                  terms.terms.len(),
                              );
                              for term in terms.terms.into_iter() {
                                  println!("- {term}");
                              }
                        }
                        None => {
                            println!("No such index kind: {kind:?}");
                        }
                    }
                }
                None => {
                    let kinds = platform.pc_platform.list_kinds().await?;
                    println!("There are total of {} kind(s) of indexes registered, they are:", kinds.len());
                    for kind in kinds.into_iter() {
                        println!("- {kind}");
                    }
                }
            }
        }
    }
    Ok(())
}

async fn parse_link_cmd(
    platform: &Platform,
    arg: LinkCmd,
) -> anyhow::Result<()> {
    match arg {
        LinkCmd::Create { resource_path, kind, terms } => {
            platform.pc_platform.resource_link_kind_with_terms(
                &resource_path,
                &kind,
                &mut terms.iter()
                    .map(String::as_ref),
            ).await?;
            println!("indexed {resource_path:?}, {kind:?}, {terms:?}");
        }
        LinkCmd::List { resource_path } => {
            let results = platform.pc_platform.get_resource_kinded_terms(
                &resource_path,
            ).await?;
            println!("Resource at path {resource_path:?} contains the following kinded terms:");
            let output = serde_json::to_string_pretty(&results)?;
            println!("{output}");
        }
        LinkCmd::Forget { resource_path, kind } => {
            platform.pc_platform.forget_resource_path(
                kind.as_deref(),
                &resource_path,
            ).await?;
            println!("forgotten {resource_path:?}");
        }
    }
    Ok(())
}

async fn parse_query_cmd(
    platform: &Platform,
    QueryCmd { kind, term }: QueryCmd,
) -> anyhow::Result<()> {
    match platform.pc_platform.list_resources(
        &kind,
        &term,
    ).await? {
        Some(results) => {
            println!(
                "Querying index of kind {kind:?} for term {term:?} got {} result(s)",
                results.resource_paths.len()
            );
            for resource_path in results.resource_paths.iter() {
                println!("- {resource_path}");
            }
        }
        None => {
            println!("No index of kind {kind:?} or term {term:?}");
        }
    }
    Ok(())
}
