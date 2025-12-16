use crate::command::Command;

mod command;
mod commands;
mod repl;
mod utils;

#[tokio::main]
async fn main() {
    let entrypoint = commands::parse();
    entrypoint
        .execute()
        .await
        .unwrap_or_else(|error| eprintln!("{}", error));
}
