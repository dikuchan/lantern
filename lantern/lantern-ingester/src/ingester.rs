use std::path::{Path, PathBuf};
use std::sync::Arc;

use arrow_json::reader::infer_json_schema;
use arrow_json::ReaderBuilder;
use parquet::arrow::AsyncArrowWriter;
use parquet::basic::{Compression, ZstdLevel};
use parquet::file::properties::WriterProperties;
use tokio::fs::{self, File};
use tokio::io::{AsyncBufReadExt, AsyncRead, BufReader};
use uuid::Uuid;

#[derive(Debug, thiserror::Error)]
pub enum IngestError {
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Arrow Error: {0}")]
    Arrow(#[from] arrow::error::ArrowError),
    #[error("Parquet Error: {0}")]
    Parquet(#[from] parquet::errors::ParquetError),
}

pub struct Ingester {
    table_name: String,
    data_dir_path: PathBuf,
    batch_size_line_count: usize,
}

impl Ingester {
    pub fn new<P: AsRef<Path>>(table_name: &str, data_dir_path: P) -> Self {
        Self {
            table_name: table_name.to_owned(),
            data_dir_path: data_dir_path.as_ref().to_owned(),
            batch_size_line_count: 50_000,
        }
    }

    pub async fn ingest<R>(&self, reader: R) -> Result<(), IngestError>
    where
        R: AsyncRead + Unpin,
    {
        let table_dir_path = self.data_dir_path.join(&self.table_name);
        fs::create_dir_all(&table_dir_path).await?;

        let mut lines = BufReader::new(reader).lines();
        let mut buffer: Vec<String> = Vec::with_capacity(self.batch_size_line_count);

        while let Some(line) = lines.next_line().await? {
            if !line.trim().is_empty() {
                buffer.push(line);
            }

            if buffer.len() >= self.batch_size_line_count {
                println!("Flushing '{}' logs", buffer.len());
                self.flush_batch(&table_dir_path, &buffer).await?;
                buffer.clear();
            }
        }

        if !buffer.is_empty() {
            println!("Flushing remaining '{}' logs", buffer.len());
            self.flush_batch(&table_dir_path, &buffer).await?;
        }

        Ok(())
    }

    async fn flush_batch(
        &self,
        table_dir_path: &Path,
        buffer: &[String],
    ) -> Result<(), IngestError> {
        if buffer.is_empty() {
            return Ok(());
        }

        let payload = buffer.join("\n");
        let cursor = std::io::Cursor::new(payload.as_bytes());

        let mut buffer_reader = std::io::BufReader::new(payload.as_bytes());
        let (inferred_schema, size) = infer_json_schema(&mut buffer_reader, None)?;

        let mut reader = ReaderBuilder::new(Arc::new(inferred_schema))
            .with_batch_size(size)
            .build(cursor)?;

        let batch = match reader.next() {
            Some(Ok(b)) => b,
            Some(Err(e)) => return Err(IngestError::Arrow(e)),
            None => return Ok(()),
        };

        let file_name = format!("{}.parquet", Uuid::new_v4().to_string());
        let file_path = table_dir_path.join(file_name);

        let file = File::create(&file_path).await?;

        let writer_properties = WriterProperties::builder()
            .set_compression(Compression::ZSTD(ZstdLevel::default()))
            .build();

        let mut writer = AsyncArrowWriter::try_new(file, batch.schema(), Some(writer_properties))?;
        writer.write(&batch).await?;
        writer.close().await?;

        Ok(())
    }
}
