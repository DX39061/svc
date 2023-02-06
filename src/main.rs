mod cli;
mod bucket;
mod remote;
mod util;

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

        Some(Commands::Status {}) => status(),

        Some(Commands::Commit { message }) => commit(message),

        Some(Commands::Checkout { version }) => checkout(version),

        Some(Commands::Pull {}) => pull(),

        Some(Commands::Push {}) => push(),

        Some(Commands::SetRemote { url }) => set_remote(url),

        None => info()
    }
}
