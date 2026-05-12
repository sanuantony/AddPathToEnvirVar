use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(name = "pathctl")]
#[command(about = "Safe PATH editor")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum Scope {
    User,
    System,
}

impl Scope {
    pub fn is_system(self) -> bool {
        matches!(self, Scope::System)
    }
}

#[derive(Subcommand)]
pub enum Commands {
    List {
        #[arg(long, value_enum, default_value_t = Scope::User)]
        scope: Scope,
    },
    Add {
        path: String,
        #[arg(long, value_enum, default_value_t = Scope::User)]
        scope: Scope,
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        yes: bool,
    },
    Remove {
        path: String,
        #[arg(long, value_enum, default_value_t = Scope::User)]
        scope: Scope,
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        yes: bool,
    },
    Backup {
        file: String,
        #[arg(long, value_enum, default_value_t = Scope::User)]
        scope: Scope,
    },
    Restore {
        file: String,
        #[arg(long, value_enum, default_value_t = Scope::User)]
        scope: Scope,
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        yes: bool,
    },
}