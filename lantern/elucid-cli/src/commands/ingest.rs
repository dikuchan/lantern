use std::path::PathBuf;

use anyhow::anyhow;
use clap::Args;
use elucid_ingester::Ingester;
use tokio::{fs, io};

use crate::command::Command;
use crate::utils::get_data_dir_path;

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
        let data_dir_path = get_data_dir_path(self.data_dir_path.clone())?;
        fs::create_dir_all(&data_dir_path).await?;

        let data_dir_path = data_dir_path.to_str().ok_or(anyhow!("Path is not UTF-8"))?;
        let ingester = Ingester::new(&self.table, data_dir_path);

        ingester.ingest(io::stdin()).await?;

        Ok(())
    }
}
