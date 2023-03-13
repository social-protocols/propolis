mod auth;
mod db;
mod error;
mod pages;
mod static_path;
mod structs;
mod util;

use pages::index::index;
use pages::merge::{merge, merge_post};
use pages::new_statement::new_statement;
use pages::options::{options, options_post};
use pages::statement::statement_page;
use pages::submissions::submissions;

use tower_http::compression::CompressionLayer;

use axum::{
    routing::{get, post},
    Extension, Router,
};
use tower_cookies::CookieManagerLayer;
use tower_http::trace::TraceLayer;

use std::net::SocketAddr;

use include_dir::{include_dir, Dir};

use crate::db::setup_db;
use crate::pages::new_statement::completions;
use crate::pages::new_statement::create_statement;
use crate::pages::statement::votes;
use crate::pages::vote::vote;
use crate::static_path::static_path;

// embed files in /static into the binary
static STATIC_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/static");

#[tokio::main]
async fn main() {
    let sqlite_pool = setup_db().await;

    // Setup tracing
    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/", get(index))
        .route("/vote", post(vote))
        .route("/completions", post(completions))
        .route("/statement/:id", get(statement_page))
        .route("/votes/:id", get(votes))
        .route("/merge/:secret", get(merge))
        .route("/merge/:secret", post(merge_post))
        .route("/new", get(new_statement))
        .route("/create", post(create_statement))
        .route("/options", get(options))
        .route("/options", post(options_post))
        .route("/submissions", get(submissions))
        .route("/*path", get(static_path))
        .layer(TraceLayer::new_for_http())
        .layer(Extension(sqlite_pool))
        .layer(CookieManagerLayer::new())
        .layer(CompressionLayer::new());

    let addr = SocketAddr::from(([0, 0, 0, 0], 8000));
    println!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
