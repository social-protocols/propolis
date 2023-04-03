mod auth;
mod db;
mod error;
mod pages;
mod prediction;

mod static_handler;

mod structs;
mod util;

use axum::response::Html;
use http::StatusCode;
use pages::index::index;
use pages::merge::{merge, merge_post};
use pages::new_statement::link_followup;
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
use crate::pages::new_statement::completions;
use crate::pages::new_statement::create_statement;
use crate::pages::statement::votes;
use crate::pages::subscribe::subscribe;
use crate::pages::vote::vote;

// embed static files into release binary
#[derive(RustEmbed)]
#[folder = "static/"]
struct StaticAsset;

#[tokio::main]
async fn main() {
    let sqlite_pool = setup_db().await;

    // Setup tracing
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    let mut app = Router::new()
        .route("/", get(index))
        .route("/vote", post(vote))
        .route("/subscribe", post(subscribe))
        .route("/completions", post(completions))
        .route("/statement/:id", get(statement_page))
        .route("/votes/:id", get(votes))
        .route("/merge/:secret", get(merge))
        .route("/merge/:secret", post(merge_post))
        .route("/new", get(new_statement))
        .route("/create", post(create_statement))
        .route("/link_followup", post(link_followup))
        .route("/options", get(options))
        .route("/options", post(options_post))
        .route("/subscriptions", get(subscriptions))
        .route("/*file", get(static_handler::static_handler))
        .layer(TraceLayer::new_for_http())
        .layer(Extension(sqlite_pool.to_owned()))
        .layer(CookieManagerLayer::new())
        .layer(CompressionLayer::new())
        .fallback_service(get(not_found));

    if cfg!(feature = "with_predictions") {
        app = app.route(
            "/prediction/:id",
            get(crate::pages::prediction::prediction_page),
        );
    }

    let prediction_runner = prediction::runner::run(&sqlite_pool);
    let addr = SocketAddr::from(([0, 0, 0, 0], 8000));
    info!("listening on {}", addr);
    let axum_server = axum::Server::bind(&addr).serve(app.into_make_service());

    let (_, axum_result) = futures::future::join(prediction_runner, axum_server).await;
    axum_result.unwrap();
}

async fn not_found() -> (StatusCode, Html<String>) {
    (StatusCode::NOT_FOUND, Html("Not found".to_string()))
}
