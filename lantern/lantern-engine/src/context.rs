use crate::planner::QueryPlanner;

use datafusion::error::{DataFusionError, Result};
use datafusion::prelude::*;
use datafusion::prelude::{DataFrame, SessionConfig, SessionContext};
use lantern_language::parser;

pub struct Context {
    context: SessionContext,
}

impl Context {
    pub fn new() -> Self {
        let config = SessionConfig::new().with_information_schema(true);
        let context = SessionContext::new_with_config(config);
        Self { context }
    }

    pub async fn register_csv(&self, table_name: &str, table_path: &str) -> Result<()> {
        let options = CsvReadOptions::new().has_header(true);
        self.context
            .register_csv(table_name, table_path, options)
            .await?;
        Ok(())
    }

    pub async fn execute(&self, source: &str) -> Result<DataFrame> {
        let query = parser::parse(source).map_err(|error| {
            match error.eprint(source) {
                Ok(()) => {}
                Err(error) => return DataFusionError::IoError(error),
            };
            DataFusionError::Plan(format!("Parse error: {:?}", error))
        })?;

        let planner = QueryPlanner::new(&self.context);
        let plan = planner.create_logical_plan(query).await?;

        self.context
            .execute_logical_plan(plan)
            .await
            .map_err(|error| error.into())
    }
}
