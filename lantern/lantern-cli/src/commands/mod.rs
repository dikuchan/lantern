mod entrypoint;
mod execute;
mod ingest;
mod validate;

use clap::Parser;

use self::entrypoint::Entrypoint;
use self::execute::ExecuteCommand;
use self::ingest::IngestCommand;
use self::validate::ValidateCommand;

pub fn parse() -> Entrypoint {
    Entrypoint::parse()
}
