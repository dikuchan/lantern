use std::env;
use std::path::PathBuf;

use anyhow::anyhow;
use clap::Args;
use lantern_ingester::Ingester;
use tokio::{fs, io};

use crate::command::Command;

#[derive(Args)]
pub struct IngestCommand {
    /// Name of the ingested table.
    #[arg(value_name = "TABLE")]
    pub table: String,

    /// Path to the data directory. Defaults to `$HOME/.lantern/data`.
    #[arg(long = "data-dir", short = 'd', value_name = "DATA_DIR")]
    pub data_dir_path: Option<PathBuf>,
}

impl Command for IngestCommand {
    async fn execute(&self) -> anyhow::Result<()> {
        let home_dir_path = env::home_dir().ok_or(anyhow!("Cannot access home directory"))?;
        let data_dir_path = home_dir_path.join(".lantern").join("data");
        fs::create_dir_all(&data_dir_path).await?;

        let data_dir_path = data_dir_path.to_str().ok_or(anyhow!("Path is not UTF-8"))?;
        let ingester = Ingester::new(&self.table, data_dir_path);

        ingester.ingest(io::stdin()).await?;

        Ok(())
    }
}
