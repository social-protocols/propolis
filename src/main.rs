mod auth;
mod db;
mod error;
mod highlight;
mod opts;
mod pages;
mod prediction;

mod static_handler;

mod structs;
mod util;

use axum::response::Html;
use clap::Parser;
use http::StatusCode;
use pages::index::index;
use pages::merge::{merge, merge_post};
use pages::new_statement::new_statement;
use pages::options::{options, options_post};
use pages::statement::statement_page;
use pages::subscriptions::subscriptions;

use rust_embed::RustEmbed;
use tower_http::compression::CompressionLayer;

use axum::{
    routing::{get, post},
    Extension, Router,
};
use tower_cookies::CookieManagerLayer;
use tower_http::trace::TraceLayer;
use tracing::info;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

use std::net::SocketAddr;

use crate::db::setup_db;
use crate::opts::ProgramOpts;
use crate::pages::itdepends::itdepends;
use crate::pages::itdepends::itdepends_completions;
use crate::pages::itdepends::itdepends_create;
use crate::pages::new_statement::create_statement;
use crate::pages::new_statement::new_statement_completions;
use crate::pages::subscribe::subscribe;
use crate::pages::user::user_page;
use crate::pages::vote::vote;

// embed static files into release binary
#[derive(RustEmbed)]
#[folder = "static/"]
struct StaticAsset;

#[tokio::main]
async fn main() {
    let opts = ProgramOpts::parse();
    let mut sqlite_pool = setup_db(&opts.database).await;

    // Setup tracing
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    let mut app = Router::new();

    #[cfg(feature = "with_predictions")]
    {
        app = app.route(
            "/prediction/:id",
            get(crate::pages::prediction::prediction_page),
        );
    }

    app = app
        .route("/", get(index))
        .route("/vote", post(vote))
        .route("/subscribe", post(subscribe))
        .route("/user", get(user_page))
        .route("/statement/:id", get(statement_page))
        .route("/merge/:secret", get(merge))
        .route("/merge/:secret", post(merge_post))
        .route("/new", get(new_statement))
        .route("/new/completions", post(new_statement_completions))
        .route("/statement/:id/itdepends", get(itdepends))
        .route("/statement/:id/itdepends", post(itdepends_create))
        .route("/itdepends_completions", post(itdepends_completions))
        .route("/create", post(create_statement))
        .route("/options", get(options))
        .route("/options", post(options_post))
        .route("/subscriptions", get(subscriptions))
        .route("/*file", get(static_handler::static_handler))
        .layer(TraceLayer::new_for_http())
        .layer(Extension(sqlite_pool.to_owned()))
        .layer(CookieManagerLayer::new())
        .layer(CompressionLayer::new())
        .fallback_service(get(not_found));

    let prediction_runner = prediction::runner::run(opts.prediction, &mut sqlite_pool);
    let addr = SocketAddr::from(([0, 0, 0, 0], 8000));
    info!("listening on {}", addr);
    let axum_server = axum::Server::bind(&addr).serve(app.into_make_service());

    let (_, axum_result) = futures::future::join(prediction_runner, axum_server).await;
    axum_result.unwrap();
}

async fn not_found() -> (StatusCode, Html<String>) {
    (StatusCode::NOT_FOUND, Html("Not found".to_string()))
}
