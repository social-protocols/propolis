mod api;
mod auth;
mod command_line_args;
mod db;
mod db_setup;
mod error;
mod highlight;
mod pages;
mod prediction;

mod http_server;
mod http_static;

mod structs;
mod util;

use clap::Parser;
use http_server::start_http_server;

use anyhow::{Context, Result};

use tracing_subscriber::{fmt, prelude::*, EnvFilter};

use crate::db_setup::setup_database;
use crate::command_line_args::CommandLineArgs;

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing();

    let command_line_args = CommandLineArgs::parse();
    let sqlite_pool = setup_database(&command_line_args.database).await;
    let mut sqlite_pool_prediction_runner = sqlite_pool.clone();

    // depending on the feature flags, the pool needs a mutable reference or not
    tokio::select! {
        res = start_http_server(sqlite_pool.clone()) => {
            res.context("http server crashed").unwrap();
        }

        res = {
            #[allow(clippy::unnecessary_mut_passed)]
            prediction::runner::run(&command_line_args.prediction, &mut sqlite_pool_prediction_runner)
        } => {
            res.context("prediction runner crashed").unwrap();
        }

    }

    Ok(())
}

fn init_tracing() {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();
}
