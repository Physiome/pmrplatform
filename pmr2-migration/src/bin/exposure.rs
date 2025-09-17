use std::{
    collections::HashMap,
    fs::File,
    io::{BufReader, stdin},
    path::PathBuf,
};
use clap::{Parser, Subcommand};
use pmrctrl::platform::Builder;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct Entry(String, Record);

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
enum Record {
    // the top exposure record
    Top(Top),
    // definition per file
    ExposureFile(ExposureFile),
}

#[derive(Debug, Deserialize, Serialize)]
struct Top {
    commit_id: String,
    title: String,
    workspace: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct ExposureFile {
    views: Vec<View>,
    file_type: String,
    selected_view: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct View(String, Option<HashMap<String, Option<String>>>);

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
        Commands::Import { input } => {
            let entries: Vec<Entry> = match input {
                Some(path) => serde_json::from_reader(BufReader::new(File::open(path)?))?,
                None => serde_json::from_reader(BufReader::new(stdin()))?,
            };

            let mut top = None::<Top>;
            let mut files = Vec::<ExposureFile>::new();
            for Entry(target, record) in entries.into_iter() {
                match record {
                    Record::Top(t) => {
                        top.replace(t);
                    }
                    Record::ExposureFile(ef) => {
                        files.push(ef);
                    }
                }
            }
            // with the top, create exposure
            // with the files, create exposure files in the exposure
        },
    }

    Ok(())
}
