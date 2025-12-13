use crate::command::Command;
use crate::commands::validate::ValidateCommand;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about)]
pub struct Entrypoint {
    #[command(subcommand)]
    subcommand: Option<Subcommands>,
}

impl Command for Entrypoint {
    fn execute(&self) -> anyhow::Result<()> {
        match &self.subcommand {
            Some(Subcommands::Validate(command)) => command.execute(),
            None => Ok(()),
        }
    }
}

#[derive(Subcommand)]
pub enum Subcommands {
    Validate(ValidateCommand),
}
