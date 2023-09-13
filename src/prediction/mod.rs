#[cfg(feature = "with_predictions")]
pub mod data;

#[cfg(feature = "with_predictions")]
pub mod embedding;

#[cfg(feature = "with_predictions")]
pub mod multi_statement_classifier;

#[cfg(feature = "with_predictions")]
pub mod prompts;

#[cfg(feature = "with_predictions")]
pub mod runner;

#[cfg(not(feature = "with_predictions"))]
pub mod runner {
    use anyhow::Result;
    use sqlx::SqlitePool;

    pub async fn run(
        _args: &crate::command_line_args::PredictionArgs,
        _pool: &SqlitePool,
    ) -> Result<()> {
        Ok(())
    }
}
