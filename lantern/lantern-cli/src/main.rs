use crate::command::Command;

mod command;
mod commands;

fn main() {
    let entrypoint = commands::parse();
    entrypoint
        .execute()
        .unwrap_or_else(|error| eprintln!("{}", error));
}
