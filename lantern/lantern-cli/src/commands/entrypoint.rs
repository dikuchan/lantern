use crate::command::Command;
use crate::commands::execute::ExecuteCommand;
use crate::commands::validate::ValidateCommand;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about)]
pub struct Entrypoint {
    #[command(subcommand)]
    subcommand: Option<Subcommands>,
}

impl Command for Entrypoint {
    async fn execute(&self) -> anyhow::Result<()> {
        match &self.subcommand {
            Some(Subcommands::Validate(v)) => v.execute().await,
            Some(Subcommands::Execute(v)) => v.execute().await,
            None => Ok(()),
        }
    }
}

#[derive(Subcommand)]
pub enum Subcommands {
    Validate(ValidateCommand),
    Execute(ExecuteCommand),
}
