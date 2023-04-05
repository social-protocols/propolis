use clap::Parser;

#[cfg(feature = "with_predictions")]
#[derive(Parser, Clone, Debug)]
pub struct PredictionOpts {
    /// Used (openai) tokens allowed per duration before rate limiting takes action
    #[arg(long, env, default_value_t = 1000)]
    pub tokens_per_duration: u64,
    /// Duration length in seconds for rate limiting used (openai) tokens
    #[arg(long, env, default_value_t = 60)]
    pub seconds_per_duration: u64,
}

/// Program options to be read via clap
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct ProgramOpts {
    #[cfg(feature = "with_predictions")]
    #[command(flatten)]
    pub prediction: PredictionOpts,
}
