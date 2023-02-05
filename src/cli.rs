use clap::{Parser, Subcommand, Args};

/// Single-line Verion Control System
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// show repo info
    Info {},

    /// initialize a svc repo
    Init {},

    /// show all versions log
    Log {},

    /// save current workplace
    Commit {
        /// commit message
        #[arg(short, long)]
        message: String
    },

    /// switch to specific version
    Checkout { version: String },

    /// push to remote repo
    Push {},

    /// pull from remote repo
    Pull {},

    /// set remote repo url
    SetRemote { url: String },
}
