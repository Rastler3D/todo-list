use clap::Parser;
use crate::cli::Cli;
use crate::command::CommandError;

mod task;
mod cli;
mod query;
mod storage;
mod command;

fn main() -> Result<(), CommandError> {
    Cli::parse().run()
}
