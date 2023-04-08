use clap::Parser;

#[cfg(feature = "with_predictions")]
#[derive(Parser, Clone, Debug)]
pub struct PredictionOpts {
    /// API key used for requests to openai
    #[arg(long, env)]
    pub openai_api_key: Option<String>,
    /// API keys used for requests to openai delimited via ":"
    #[arg(long, env, value_parser, num_args=1.., value_delimiter=':')]
    pub openai_api_keys: Vec<String>,
    /// Used (openai) tokens allowed per duration before rate limiting takes action
    #[arg(long, env, default_value_t = 1000)]
    pub tokens_per_duration: u64,
    /// Duration length in seconds for rate limiting used (openai) tokens
    #[arg(long, env, default_value_t = 60)]
    pub seconds_per_duration: u64,
}
#[cfg(not(feature = "with_predictions"))]
#[derive(Parser, Clone, Debug)]
pub struct PredictionOpts {}

#[derive(Parser, Clone, Debug)]
pub struct DatabaseOpts {
    /// URL to database
    #[arg(long, env, required = true)]
    pub database_url: String,
}

/// Program options to be read via clap
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct ProgramOpts {
    #[command(flatten)]
    pub prediction: PredictionOpts,
    #[command(flatten)]
    pub database: DatabaseOpts,
}
