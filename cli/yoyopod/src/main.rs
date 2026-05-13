use std::process::ExitCode;

use anyhow::Result;
use clap::Parser;

mod cli;
mod commands;
mod deploy_config;
mod local;
mod logging;
mod paths;
mod quoting;
mod repo;
mod ssh;

fn main() -> ExitCode {
    let args = cli::Cli::parse();
    logging::init(args.verbose);
    match run(args) {
        Ok(code) => ExitCode::from(code as u8),
        Err(err) => {
            eprintln!("error: {err:#}");
            ExitCode::from(1)
        }
    }
}

fn run(args: cli::Cli) -> Result<i32> {
    match args.command {
        cli::Command::Target(t) => commands::target::dispatch(t, args.dry_run),
    }
}
