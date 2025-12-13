mod entrypoint;
mod validate;

use entrypoint::Entrypoint;

use clap::Parser;

pub fn parse() -> Entrypoint {
    Entrypoint::parse()
}
