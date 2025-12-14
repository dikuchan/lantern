use crate::command::Command;

use std::fs::File;
use std::io::{stdin, Read};
use std::path::PathBuf;

use anyhow::anyhow;
use clap::Args;
use lantern_engine::Context;

#[derive(Args)]
pub struct ExecuteCommand {
    /// Name of a table in the query.
    #[arg(value_name = "TABLE")]
    pub table_name: String,

    /// Path to a table file.
    #[arg(value_name = "TABLE_FILE")]
    pub table_file_path: PathBuf,

    /// Query text.
    #[arg(value_name = "QUERY")]
    pub source: Option<String>,

    /// Path to a query file.
    #[arg(long = "file", short = 'f', value_name = "FILE")]
    pub file_path: Option<PathBuf>,
}

impl ExecuteCommand {
    async fn execute_query_input<R: Read>(&self, mut input: R) -> anyhow::Result<()> {
        let mut buffer = Vec::new();
        let _ = input.read_to_end(&mut buffer)?;
        let source = String::from_utf8(buffer)?;

        self.execute_query(
            &source,
            &self.table_name,
            self.table_file_path
                .to_str()
                .ok_or(anyhow!("Path is not UTF-8"))?,
        )
        .await
    }

    async fn execute_query(
        &self,
        source: &str,
        table_name: &str,
        table_file_path: &str,
    ) -> anyhow::Result<()> {
        let context = Context::new();
        context.register_csv(table_name, table_file_path).await?;

        let data = context.execute(&source).await?;
        data.show().await?;

        Ok(())
    }
}

impl Command for ExecuteCommand {
    async fn execute(&self) -> anyhow::Result<()> {
        match &self.source {
            Some(source) => self.execute_query_input(source.as_bytes()).await,
            None => match &self.file_path {
                Some(file_path) => {
                    let file = File::open(file_path)?;
                    self.execute_query_input(&file).await
                }
                None => self.execute_query_input(stdin()).await,
            },
        }
    }
}
