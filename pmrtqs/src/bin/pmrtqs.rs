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
use pmrmodel::model::db::task_template::TaskTemplateBackend;
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
    Arg {
        #[command(subcommand)]
        arg: Arg,
    },
    #[command(arg_required_else_help = true)]
    Choice {
        #[command(subcommand)]
        choice: Choice,
    }
}

#[derive(Debug, Subcommand)]
enum Arg {
    #[command(arg_required_else_help = true)]
    Add {
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
    #[command(arg_required_else_help = true)]
    Rm {
        #[arg(long, value_name = "ARG_ID")]
        argid: i64,
    },
    #[command(arg_required_else_help = true)]
    Show {
        id: i64,
    },
}

#[derive(Debug, Subcommand)]
enum Choice {
    #[command(arg_required_else_help = true)]
    Add {
        #[arg(long, value_name = "ARG_ID")]
        argid: i64,
        label: String,
        value: Option<String>,
    },
    #[command(arg_required_else_help = true)]
    Rm {
        #[arg(long, value_name = "CHOICE_ID")]
        choiceid: i64,
    },
    #[command(arg_required_else_help = true)]
    Show {
        #[arg(long, value_name = "ARG_ID")]
        argid: i64,
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
        Commands::Arg { arg } => {
            parse_arg(arg, &backend).await?;
        }
        Commands::Choice { choice } => {
            parse_choice(choice, &backend).await?
        }
    }

    Ok(())
}


async fn parse_arg(arg: Arg, backend: &SqliteBackend) -> anyhow::Result<()> {
    match arg {
        Arg::Add { id, flag, flag_joined, prompt, default_value, choice_fixed, choice_source } => {
            println!("Setting argument for id {}", &id);
            TaskTemplateBackend::add_task_template_arg(
                backend,
                id,
                flag.as_deref(),
                flag_joined,
                prompt.as_deref(),
                default_value.as_deref(),
                choice_fixed,
                choice_source.as_deref(),
            ).await?;
            let task_template = TaskTemplateBackend::get_task_template_by_id(
                backend, id,
            ).await?;
            println!("{}", task_template);
        }
        Arg::Show { id } => {
            let task_template = TaskTemplateBackend::get_task_template_by_id(
                backend, id,
            ).await?;
            let args = task_template.args.unwrap_or([].into());
            println!(
                "Showing detailed arguments for task template id:{}",
                task_template.id,
            );
            for arg in args.iter() {
                println!("arg id:{}> {}", arg.id, arg);
            }
        }
        Arg::Rm { argid } => {
            let result = TaskTemplateBackend::delete_task_template_arg_by_id(
                backend, argid).await?;
            match result {
                None => {
                    match TaskTemplateBackend::get_task_template_by_arg_id(
                        backend, argid,
                    ).await {
                        Ok(task_template) => {
                            match task_template.final_task_template_arg_id {
                                Some(_) => {
                                    println!("task template already finalized");
                                    println!("{}", task_template);
                                }
                                None => {
                                    println!("task template not finalized but failed to remove");
                                }
                            }
                        }
                        Err(_) => println!("no argument with argument id:{}", argid),
                    }
                }
                Some(arg) => {
                    println!("argument id:{} deleted: {}", argid, arg);
                    let task_template = TaskTemplateBackend::get_task_template_by_id(
                        backend, arg.task_template_id,
                    ).await?;
                    println!("{}", task_template);
                }
            }
        }
    };
    Ok(())
}

async fn parse_choice(choice: Choice, backend: &SqliteBackend) -> anyhow::Result<()> {
    match choice {
        Choice::Add { argid, label, value } => {
            println!("Adding choice for arg:id {}", &argid);
            TaskTemplateBackend::add_task_template_arg_choice(
                backend,
                argid,
                value.as_deref(),
                &label,
            ).await?;
        }
        Choice::Rm { choiceid } => {
            println!("removing choice with choice:id {}", &choiceid);
            TaskTemplateBackend::delete_task_template_arg_choice_by_id(
                backend,
                choiceid,
            ).await?;
        }
        Choice::Show { argid } => {
            match TaskTemplateBackend::get_task_template_arg_by_id(
                backend,
                argid,
            ).await? {
                Some(arg) => {
                    println!("arg id:{}> {}", arg.id, arg);
                    match arg.choices.as_ref() {
                        None => println!("<choices not selected with template>"),
                        Some(choices) => {
                            if choices.iter().peekable().peek().is_none() {
                                println!("<no choices recorded>");
                            }
                            for choice in choices.iter() {
                                println!("  choice id:{}> {}", choice.id, choice);
                            }
                        }
                    }
                }
                // this should error?
                None => println!("no argument with argid: {}", argid),
            };
        }
    };
    Ok(())
}
