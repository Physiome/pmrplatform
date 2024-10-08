use clap::{
    Parser,
    Subcommand,
};
use pmrac::{
    password::Password,
    platform::{
        Builder as PlatformBuilder,
        Platform,
    },
};
use pmrcore::ac::{
    agent::Agent,
    role::Role,
    workflow::State,
};
use pmrmodel::backend::db::{
    MigrationProfile,
    SqliteBackend,
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
    #[command(arg_required_else_help = true)]
    Role {
        #[command(subcommand)]
        cmd: RoleCmd,
    },
    #[command(arg_required_else_help = true)]
    Resource {
        resource: String,
        #[command(subcommand)]
        cmd: ResourceCmd,
    },
}

#[derive(Debug, Subcommand)]
enum UserCmd {
    #[command(arg_required_else_help = true)]
    Create {
        name: String,
    },
    #[command(arg_required_else_help = true)]
    Password {
        name: String,
        #[command(subcommand)]
        cmd: PasswordCmd,
    },
    #[command(arg_required_else_help = true)]
    Status {
        name: String,
    },
}

#[derive(Debug, Subcommand)]
enum PasswordCmd {
    /// Reports the status of the password
    Check,
    /// Force a password reset
    Reset,
    /// Restrict the user account
    Restrict,
    #[command(arg_required_else_help = true)]
    /// Sets the password for the user
    Set {
        password: String,
    },
}

#[derive(Debug, Subcommand)]
enum RoleCmd {
    #[command(arg_required_else_help = true)]
    Grant {
        login: String,
        #[arg(value_enum)]
        role: Role,
    },
    #[command(arg_required_else_help = true)]
    Revoke {
        login: String,
        #[arg(value_enum)]
        role: Role,
    },
}

#[derive(Debug, Subcommand)]
enum ResourceCmd {
    #[command(arg_required_else_help = true)]
    Role {
        #[command(subcommand)]
        cmd: RoleCmd,
    },
    // use workflow transition instead whenever that gets implemented.
    // for now just provide a way to set the state directly
    #[command(arg_required_else_help = true)]
    State {
        #[arg(value_enum)]
        state: State,
    },
    Status,
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
    let platform = PlatformBuilder::new()
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
        Commands::Role { cmd } => {
            parse_role(&platform, cmd).await?;
        },
        Commands::Resource { resource, cmd } => {
            parse_resource(&platform, resource, cmd).await?;
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
        UserCmd::Password { name, cmd } => {
            parse_password(&platform, name, cmd).await?
        }
        UserCmd::Status { name } => {
            use pmrcore::ac::user::User;

            let (
                User { id, name, created_ts },
                password_status
            ) = platform.login_status(&name).await?;
            println!("id: {id}");
            println!("name: {name}");
            println!("created_ts: {created_ts}");
            println!("status: {password_status}");

            // could have not destructured it but getting it done this way for now...
            let agent = Agent::User(User { id, name, created_ts });
            let res_grants = platform.get_res_grants_for_agent(&agent).await?;
            for (res, roles) in res_grants.into_iter() {
                let role = roles.into_iter()
                    .map(<&'static str>::from)
                    .collect::<Vec<_>>()
                    .join(", ");
                println!("role(s) granted for '{res}': [{role}]");
            }
        }
    }
    Ok(())
}

async fn parse_password<'p>(
    platform: &'p Platform,
    login: String,
    arg: PasswordCmd,
) -> anyhow::Result<()> {
    match arg {
        PasswordCmd::Check => {
            let (_, status) = platform.login_status(&login).await?;
            println!("user's password status: {status:?}");
        }
        PasswordCmd::Reset => {
            let (user, _) = platform.login_status(&login).await?;
            platform.force_user_id_password(user.id, Password::Reset).await?;
            println!("forced password reset for {login} on their next login");
        }
        PasswordCmd::Restrict => {
            let (user, _) = platform.login_status(&login).await?;
            platform.force_user_id_password(user.id, Password::Restricted).await?;
            println!("restricted account for {login}");
        }
        PasswordCmd::Set { password } => {
            let (user, _) = platform.login_status(&login).await?;
            platform.force_user_id_password(user.id, Password::new(&password)).await?;
            println!("updated password for user {login}");
        }
    }
    Ok(())
}

async fn parse_role<'p>(
    platform: &'p Platform,
    arg: RoleCmd,
) -> anyhow::Result<()> {
    match arg {
        RoleCmd::Grant { login, role } => {
            let (user, _) = platform.login_status(&login).await?;
            if platform.grant_role_to_user(user, role).await? {
                println!("role {role} granted to {login}");
            } else {
                println!("role {role} was already granted to {login}");
            }
        }
        RoleCmd::Revoke { login, role } => {
            let (user, _) = platform.login_status(&login).await?;
            if platform.revoke_role_from_user(user, role).await? {
                println!("role {role} revoked from {login}");
            } else {
                println!("{login} has no role {role} to be revoked");
            }
        }
    }
    Ok(())
}

async fn parse_resource<'p>(
    platform: &'p Platform,
    resource: String,
    arg: ResourceCmd,
) -> anyhow::Result<()> {
    match arg {
        ResourceCmd::Role { cmd } => {
            parse_resource_role(&platform, resource, cmd).await?
        }
        ResourceCmd::State { state } => {
            platform.set_wf_state_for_res(&resource, state).await?;
            println!("workflow state for resource {resource} set to {state}");
        }
        ResourceCmd::Status => {
            let state = platform.get_wf_state_for_res(&resource).await?;
            println!("workflow state for resource {resource} is: {state}");
            let res_grants = platform.get_res_grants_for_res(&resource).await?;
            for (agent, roles) in res_grants.into_iter() {
                let role = roles.into_iter()
                    .map(<&'static str>::from)
                    .collect::<Vec<_>>()
                    .join(", ");
                println!("{agent} granted role(s) [{role}]");
            }
        }
    }
    Ok(())
}

async fn parse_resource_role<'p>(
    platform: &'p Platform,
    resource: String,
    arg: RoleCmd,
) -> anyhow::Result<()> {
    match arg {
        RoleCmd::Grant { login, role } => {
            let (user, _) = platform.login_status(&login).await?;
            if platform.res_grant_role_to_agent(&resource, user, role).await? {
                println!("role {role} granted to {login} for resource {resource}");
            } else {
                println!("role {role} was already granted to {login} for resource {resource}");
            }
        }
        RoleCmd::Revoke { login, role } => {
            let (user, _) = platform.login_status(&login).await?;
            if platform.res_revoke_role_from_agent(&resource, user, role).await? {
                println!("role {role} revoked from {login} for resource {resource}");
            } else {
                println!("{login} has no role {role} for resource {resource}");
            }
        }
    }
    Ok(())
}
