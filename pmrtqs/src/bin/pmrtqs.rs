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
use pmrmodel::model::task_template::TaskTemplateBackend;
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
        version: String,
    },
    #[command(arg_required_else_help = true)]
    Finalize {
        id: i64,
    },
    #[command(arg_required_else_help = true)]
    Show {
        id: i64,
    },
    #[command(arg_required_else_help = true)]
    Args {
        id: i64,
        #[arg(long, value_name = "FLAG")]
        flag: Option<String>,
        #[arg(long, value_name = "FLAG_JOINED")]
        flag_joined: bool,
        #[arg(long, value_name = "PROMPT")]
        prompt: Option<String>,
        #[arg(long, value_name = "DEFAULT_VALUE")]
        default_value: Option<String>,
        #[arg(long, value_name = "CHOICE_FIXED")]
        choice_fixed: bool,
        #[arg(long, value_name = "CHOICE_SOURCE")]
        choice_source: Option<String>,
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
        Commands::Register { program, version } => {
            println!("registering program '{}'...", &program);
            let id = TaskTemplateBackend::add_new_task_template(
                &backend, &program, &version,
            ).await?;
            println!("program '{}' registered as id: {}", &program, id);
        }
        Commands::Finalize { id } => {
            println!("finalizing program id '{}'...", id);
            TaskTemplateBackend::finalize_new_task_template(
                &backend, id,
            ).await?;
            println!("program id {} is finalized.", id);
            let task_template = TaskTemplateBackend::get_task_template_by_id(
                &backend, id,
            ).await?;
            println!("{}", task_template);
        }
        Commands::Show { id } => {
            match TaskTemplateBackend::get_task_template_by_id(
                &backend, id,
            ).await {
                Ok(task_template) => println!("{}", task_template),
                // discern error that actually match this case
                Err(_) => println!("Task template {} not found", id),
            };
        }
        Commands::Args { id, flag, flag_joined, prompt, default_value, choice_fixed, choice_source } => {
            println!("Setting argument for id {}", &id);
            TaskTemplateBackend::add_task_template_arg(
                &backend,
                id,
                flag.as_deref(),
                flag_joined,
                prompt.as_deref(),
                default_value.as_deref(),
                choice_fixed,
                choice_source.as_deref(),
            ).await?;
            let task_template = TaskTemplateBackend::get_task_template_by_id(
                &backend, id,
            ).await?;
            println!("{}", task_template);
        }
    }

    Ok(())
}
