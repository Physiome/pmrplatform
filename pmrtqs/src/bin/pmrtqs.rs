use anyhow::bail;
use clap::{
    Parser,
    Subcommand,
};
use pmrmodel::backend::db::{
    MigrationProfile,
    SqliteBackend,
};
use pmrcore::{
    platform::TMPlatform,
    task_template::{
        TaskTemplate,
        traits::TaskTemplateBackend,
    },
};
use sqlx::{
    Sqlite,
    migrate::MigrateDatabase,
};

use pmrtqs::executor::TMPlatformExecutorInstance;


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
        #[clap(short = 'j', long = "json", action)]
        json: bool,
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
    },
    ExecOneShot,
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
    dotenvy::dotenv().ok();
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
        .run_migration_profile(MigrationProfile::Pmrtqs)
        .await?;

    match args.command {
        Commands::Register { program, version } => {
            println!("registering program '{}'...", &program);
            let (id, _) = TaskTemplateBackend::add_task_template(
                &backend, &program, &version,
            ).await?;
            println!("program '{}' registered as id: {}", &program, id);
        }
        Commands::Finalize { id } => {
            println!("finalizing program id '{}'...", id);
            let finalid = TaskTemplateBackend::finalize_new_task_template(
                &backend, id,
            ).await?;
            match finalid {
                Some(finalid) => println!("finalize with argid {finalid}."),
                None => println!("finalize failed"),
            };
            let task_template = get_task_template_by_id(&backend, id).await?;
            println!("{}", task_template);
        }
        Commands::Show { json, id } => {
            let task_template = get_task_template_by_id(&backend, id).await?;
            if json {
                println!("{}", serde_json::to_string_pretty(&task_template)?);
            } else {
                println!("{}", task_template);
            }
        }
        Commands::Arg { arg } => {
            parse_arg(arg, &backend).await?;
        }
        Commands::Choice { choice } => {
            parse_choice(choice, &backend).await?
        }
        Commands::ExecOneShot => {
            match backend
                .start_task().await?
                .map(TMPlatformExecutorInstance::from)
            {
                Some(mut executor) => {
                    executor.execute().await?;
                    println!("job completed");
                }
                None => {
                    println!("no outstanding jobs");
                }
            };
        }
    }

    Ok(())
}

async fn get_task_template_by_id(
    backend: &SqliteBackend,
    id: i64,
) -> anyhow::Result<TaskTemplate> {
    match TaskTemplateBackend::get_task_template_by_id(
        backend, id,
    ).await {
        Ok(task_template) => Ok(task_template),
        // TODO disambiguate certain problematic errors and print them
        // out.
        Err(_) => bail!("Task Template with id {} not found", id),
    }
}

async fn parse_arg(arg: Arg, backend: &SqliteBackend) -> anyhow::Result<()> {
    match arg {
        Arg::Add { id, flag, flag_joined, prompt, default_value, choice_fixed, choice_source } => {
            println!("Setting argument for task template id {id}");
            let argid = TaskTemplateBackend::add_task_template_arg(
                backend,
                id,
                flag.as_deref(),
                flag_joined,
                prompt.as_deref(),
                default_value.as_deref(),
                choice_fixed,
                choice_source.as_deref(),
            ).await?;
            println!("Created task template arg id {argid}");
            let task_template = get_task_template_by_id(
                backend, id,
            ).await?;
            println!("{}", task_template);
        }
        Arg::Show { id } => {
            let task_template = get_task_template_by_id(
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
                    let task_template = get_task_template_by_id(
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
            println!("Adding choice for arg:id {argid}");
            let choiceid = TaskTemplateBackend::add_task_template_arg_choice(
                backend,
                argid,
                value.as_deref(),
                &label,
            ).await?;
            println!("Created choice choice:id {choiceid}");
        }
        Choice::Rm { choiceid } => {
            println!("removing choice with choice:id {choiceid}");
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
