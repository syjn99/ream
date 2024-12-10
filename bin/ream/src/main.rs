use clap::Parser;

use ream::cli::{Cli, Commands};

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Node(cmd) => {
            println!("Starting node with verbosity {}", cmd.verbosity);
        }
    }
}
