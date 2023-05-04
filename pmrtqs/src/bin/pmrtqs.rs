use clap::{
    Args,
    Parser,
    Subcommand,
    ValueEnum,
};
use pmrmodel::backend::db::{
    Profile,
    SqliteBackend,
};
use sqlx::{
    Sqlite,
    migrate::MigrateDatabase,
};
use std::path::PathBuf;


#[derive(Debug, Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    #[clap(short = 'v', long = "verbose", action = clap::ArgAction::Count)]
    verbose: u8,
    #[clap(long, value_name = "DATABASE_URL", env = "PMRTQS_DB_URL")]
    db_url: String,
}


#[derive(Debug, Subcommand)]
enum Commands {
    #[command(arg_required_else_help = true)]
    Register {
        program: String,
    },
    #[command(arg_required_else_help = true)]
    Args {
        id: i64,
    },
}

#[async_std::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    let args = Cli::parse();
    stderrlog::new()
        .module(module_path!())
        .verbosity((args.verbose as usize) + 1)
        .timestamp(stderrlog::Timestamp::Second)
        .init()
        .unwrap();

    if !Sqlite::database_exists(&args.db_url).await.unwrap_or(false) {
        log::warn!("database {} does not exist; creating...", &args.db_url);
        Sqlite::create_database(&args.db_url).await?
    }
    let backend = SqliteBackend::from_url(&args.db_url)
        .await?
        .run_migration_profile(Profile::Pmrtqs)
        .await?;

    match args.command {
        Commands::Register { program } => {
            println!("Registering program '{}'...", &program);
        }
        Commands::Args { id } => {
            println!("Setting argument for id {}", &id);
        }
    }

    Ok(())
}
