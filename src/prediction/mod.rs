#[cfg(feature = "with_predictions")]
pub mod data;

#[cfg(feature = "with_predictions")]
pub mod key;

#[cfg(feature = "with_predictions")]
pub mod multi_statement_classifier;

#[cfg(feature = "with_predictions")]
pub mod prompts;

#[cfg(feature = "with_predictions")]
pub mod runner;

#[cfg(not(feature = "with_predictions"))]
pub mod runner {
    use sqlx::SqlitePool;

    pub async fn run(_opts: crate::opts::PredictionOpts, _pool: &SqlitePool) {}
}
