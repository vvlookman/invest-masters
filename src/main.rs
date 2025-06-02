//! # invmst CLI

use std::env;

use clap::Parser;

use crate::cli::Commands;

mod cli;

#[derive(Parser)]
#[command(version = env!("CARGO_PKG_VERSION"))]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[tokio::main]
async fn main() {
    invmst::init().await;

    let mut args: Vec<String> = vec![];
    env::args().for_each(|arg| {
        if let Some(master_stripped) = arg.strip_prefix('@') {
            args.push("--master".to_string());
            args.push(master_stripped.to_string());
        } else {
            args.push(arg);
        }
    });

    let cli = Cli::parse_from(args);
    match &cli.command {
        Commands::Evaluate(cmd) => {
            cmd.exec().await;
        }
        Commands::Llm(cmd) => {
            cmd.exec().await;
        }
        Commands::Masters(cmd) => {
            cmd.exec().await;
        }
    }
}
