use clap::{
    Parser,
    Subcommand,
};
use pmrac::platform::{
    Builder,
    Platform,
};
use pmrmodel::{
    backend::db::{
        MigrationProfile,
        SqliteBackend,
    },
};
use pmrrbac::Builder as PmrRbacBuilder;
use sqlx::{
    Sqlite,
    migrate::MigrateDatabase,
};

#[derive(Debug, Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    #[clap(long, value_name = "PMRAC_DB_URL", env = "PMRAC_DB_URL")]
    pmrac_db_url: String,
    #[clap(short = 'v', long = "verbose", action = clap::ArgAction::Count)]
    verbose: u8,
}


#[derive(Debug, Subcommand)]
enum Commands {
    #[command(arg_required_else_help = true)]
    User {
        #[command(subcommand)]
        cmd: UserCmd,
    },
}

#[derive(Debug, Subcommand)]
enum UserCmd {
    #[command(arg_required_else_help = true)]
    Create {
        name: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    let args = Cli::parse();
    stderrlog::new()
        .module(module_path!())
        .verbosity((args.verbose as usize) + 1)
        .timestamp(stderrlog::Timestamp::Second)
        .init()
        .unwrap();

    if !Sqlite::database_exists(&args.pmrac_db_url).await.unwrap_or(false) {
        log::warn!("pmrac database {} does not exist; creating...", &args.pmrac_db_url);
        Sqlite::create_database(&args.pmrac_db_url).await?
    }
    let platform = Builder::new()
        .ac_platform(
            SqliteBackend::from_url(&args.pmrac_db_url)
                .await?
                .run_migration_profile(MigrationProfile::Pmrac)
                .await?
        )
        .pmrrbac_builder(PmrRbacBuilder::new())
        .build();

    match args.command {
        Commands::User { cmd } => {
            parse_user(&platform, cmd).await?;
        },
    }

    Ok(())
}

async fn parse_user<'p>(
    platform: &'p Platform,
    arg: UserCmd,
) -> anyhow::Result<()> {
    match arg {
        UserCmd::Create { name } => {
            let user = platform.create_user(&name).await?;
            let id = user.id();
            let name = user.name();
            println!("user {name:?} created with id {id}");
        }
    }
    Ok(())
}
