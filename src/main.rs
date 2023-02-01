mod cli;
mod bucket;
mod remote;

use bucket::*;
use clap::Parser;
use cli::*;
use remote::*;

fn main() {
    let args = Cli::parse();
    match &args.command {
        Some(Commands::Info {}) => info(),

        Some(Commands::Init {}) => init(),

        Some(Commands::Log {}) => log(),

        Some(Commands::Commit {}) => commit(),

        Some(Commands::Checkout { version }) => checkout(version),

        Some(Commands::Pull {}) => pull(),

        Some(Commands::Push {}) => push(),

        Some(Commands::SetRemote { url }) => set_remote(url),

        None => info()
    }
}
