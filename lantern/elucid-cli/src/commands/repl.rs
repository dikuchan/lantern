use std::path::PathBuf;

use anyhow::anyhow;
use clap::Args;
use elucid_engine::Context;

use crate::command::Command;
use crate::repl;
use crate::utils::get_data_dir_path;

#[derive(Args)]
pub struct ReplCommand {
    /// Path to the data directory. Defaults to `$HOME/.lantern/data`.
    #[arg(long = "data-dir", short = 'd', value_name = "DATA_DIR")]
    pub data_dir_path: Option<PathBuf>,
}

impl Command for ReplCommand {
    async fn execute(&self) -> anyhow::Result<()> {
        let data_dir_path = get_data_dir_path(self.data_dir_path.clone())?;
        if !data_dir_path.exists() {
            return Err(anyhow!("Data directory doesn't exist"));
        }

        let context = Context::new(data_dir_path);
        repl::start(&context).await?;

        Ok(())
    }
}
