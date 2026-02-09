use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "promptctl")]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Init {
        agent: String,

        #[arg(short, long, default_value = "developer")]
        role: String,

        #[arg(short, long)]
        path: Option<String>,

        #[arg(short, long)]
        force: bool,

        #[arg(long)]
        dry_run: bool,

        #[arg(long)]
        global: bool,
    },

    Show {
        language: String,

        /// Role persona (developer, senior, reviewer, security, performance, documentation, mentor, devops)
        #[arg(short, long)]
        role: Option<String>,
    },

    List,
    Clean {
        agent: String,

        #[arg(short, long)]
        path: Option<String>,
    },
}
