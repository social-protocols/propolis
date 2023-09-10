use clap::Parser;

#[cfg(feature = "with_predictions")]
#[derive(Parser, Clone, Debug)]
pub struct PredictionArgs {
    /// API key used for requests to openai
    /// Can also be passed via OPENAI_API_KEY_n
    #[arg(long, env)]
    pub openai_api_key: Option<String>,

    /// API keys used for requests to openai delimited via ":"
    /// Can also be passed via OPENAI_API_KEY_n
    #[arg(long, env, value_parser, num_args=1.., value_delimiter=':')]
    pub openai_api_keys: Vec<String>,

    /// Used (openai) tokens allowed per duration before rate limiting takes action
    #[arg(long, env, default_value_t = 1000)]
    pub tokens_per_duration: u64,

    /// Duration length in seconds for rate limiting used (openai) tokens
    #[arg(long, env, default_value_t = 60)]
    pub tokens_seconds_per_duration: u64,

    /// API calls allowed per duration
    #[arg(long, env, default_value_t = 1)]
    pub api_calls_per_duration: u64,

    /// Duration length in seconds for rate limiting API calls
    #[arg(long, env, default_value_t = 1)]
    pub api_calls_seconds_per_duration: u64,
}
#[cfg(not(feature = "with_predictions"))]
#[derive(Parser, Clone, Debug)]
pub struct PredictionOpts {}

#[derive(Parser, Clone, Debug)]
pub struct DatabaseArgs {
    /// URL to database
    #[arg(long, env, required = true)]
    pub database_url: String,
}

/// Program options to be read via clap
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct CommandLineArgs {
    #[command(flatten)]
    pub prediction: PredictionArgs,
    #[command(flatten)]
    pub database: DatabaseArgs,
}
