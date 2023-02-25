mod auth;
mod next_statement;
mod pages;
mod static_path;
mod structs;
mod util;

use pages::history::history;
use pages::index::index;
use pages::new_statement::{new_statement, new_statement_post};
use pages::options::{options, options_post};
use pages::statement::statement;
use pages::submissions::submissions;

use tower_http::compression::CompressionLayer;

use axum::{
    routing::{get, post},
    Extension, Router,
};
use std::env;
use tower_cookies::CookieManagerLayer;

use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous},
    SqlitePool,
};
use std::net::SocketAddr;

use include_dir::{include_dir, Dir};
use std::str::FromStr;

use crate::pages::vote::vote;
use crate::static_path::static_path;

async fn setup_db() -> SqlitePool {
    // high performance sqlite insert example: https://kerkour.com/high-performance-rust-with-sqlite
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    // if embed_migrations is enabled, we create the database if it doesn't exist
    let create_database_if_missing = cfg!(feature = "embed_migrations");

    let connection_options = SqliteConnectOptions::from_str(&database_url)
        .unwrap()
        .create_if_missing(create_database_if_missing)
        .journal_mode(SqliteJournalMode::Wal)
        .synchronous(SqliteSynchronous::Normal)
        .busy_timeout(std::time::Duration::from_secs(30));

    let sqlite_pool = SqlitePoolOptions::new()
        .max_connections(8)
        .acquire_timeout(std::time::Duration::from_secs(30))
        .connect_with(connection_options)
        .await
        .unwrap();

    #[cfg(feature = "embed_migrations")]
    sqlx::migrate!("./migrations")
        .run(&sqlite_pool)
        .await
        .unwrap();

    sqlx::query("pragma temp_store = memory;")
        .execute(&sqlite_pool)
        .await
        .unwrap();
    sqlx::query("pragma mmap_size = 30000000000;")
        .execute(&sqlite_pool)
        .await
        .unwrap();
    sqlx::query("pragma page_size = 4096;")
        .execute(&sqlite_pool)
        .await
        .unwrap();

    sqlite_pool
}

// embed files in /static into the binary
static STATIC_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/static");

#[tokio::main]
async fn main() {
    let sqlite_pool = setup_db().await;

    let app = Router::new()
        .route("/", get(index))
        .route("/vote", post(vote))
        .route("/statement/:id", get(statement))
        .route("/new", get(new_statement))
        .route("/new", post(new_statement_post))
        .route("/history", get(history))
        .route("/options", get(options))
        .route("/options", post(options_post))
        .route("/submissions", get(submissions))
        .route("/*path", get(static_path))
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
