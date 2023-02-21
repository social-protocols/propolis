mod auth;
mod pages;

use pages::index::{index, index_post};
use pages::new_statement::{new_statement, new_statement_post};

use auth::ensure_auth;
use axum::{
    response::{Html, Redirect},
    routing::{get, post},
    Extension, Form, Router,
};
use dotenvy::dotenv;
use std::env;
use tower_cookies::{CookieManagerLayer, Cookies};

use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous},
    SqlitePool,
};
use std::net::SocketAddr;

use askama::Template;
use chrono::Utc;
use serde::Deserialize;
use std::str::FromStr;
use timediff::*;

async fn setup_db() -> SqlitePool {
    // high performance sqlite insert example: https://kerkour.com/high-performance-rust-with-sqlite
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let connection_options = SqliteConnectOptions::from_str(&database_url)
        .unwrap()
        .create_if_missing(false)
        .journal_mode(SqliteJournalMode::Wal)
        .synchronous(SqliteSynchronous::Normal)
        .busy_timeout(std::time::Duration::from_secs(30));

    let sqlite_pool = SqlitePoolOptions::new()
        .max_connections(8)
        .acquire_timeout(std::time::Duration::from_secs(30))
        .connect_with(connection_options)
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

#[tokio::main]
async fn main() {
    dotenv().expect(".env file not found");

    let sqlite_pool = setup_db().await;

    let app = Router::new()
        .route("/", get(index))
        .route("/", post(index_post))
        .route("/new", get(new_statement))
        .route("/new", post(new_statement_post))
        .route("/history", get(history))
        .route("/submissions", get(submissions))
        .layer(Extension(sqlite_pool))
        .layer(CookieManagerLayer::new());

    let addr = SocketAddr::from(([127, 0, 0, 1], 8000));
    println!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[derive(sqlx::FromRow)]
struct VoteHistoryItem {
    statement_id: i64,
    statement_text: String,
    vote_timestamp: i64,
    vote: i64,
}

#[derive(Template)]
#[template(path = "history.j2")]
struct HistoryTemplate {
    history: Vec<VoteHistoryItem>,
}

impl HistoryTemplate {
    fn human_relative_time(&self, timestamp: &i64) -> String {
        human_relative_time(timestamp)
    }
}

fn human_relative_time(timestamp: &i64) -> String {
    let now: i64 = Utc::now().timestamp();
    let string = format!("{}s", timestamp - now); // TODO: WTF? How to calculate a relative
                                                  // duration without constructing and parsing a string?
    TimeDiff::to_diff(string).parse().unwrap()
}

async fn history(cookies: Cookies, Extension(pool): Extension<SqlitePool>) -> Html<String> {
    let user = ensure_auth(&cookies, &pool).await;
    let query =
        sqlx::query_as!(VoteHistoryItem, "select s.id as statement_id, s.text as statement_text, timestamp as vote_timestamp, vote from vote_history v join statements s on s.id = v.statement_id where user_id = ? and vote != 0 order by timestamp desc", user.id);
    let result = query.fetch_all(&pool).await.expect("Must be valid");

    let template = HistoryTemplate { history: result };

    Html(template.render().unwrap())
}

#[derive(sqlx::FromRow)]
struct SubmissionsItem {
    statement_id: i64,
    statement_text: String,
    author_timestamp: i64,
    vote: i64, // vote is nullable, should be Option<i64>, but TODO: https://github.com/djc/askama/issues/752
    yes_count: i64,
    no_count: i64,
}

#[derive(Template)]
#[template(path = "submissions.j2")]
struct SubmissionsTemplate {
    submissions: Vec<SubmissionsItem>,
}

impl SubmissionsTemplate {
    fn human_relative_time(&self, timestamp: &i64) -> String {
        human_relative_time(timestamp)
    }
}

async fn submissions(cookies: Cookies, Extension(pool): Extension<SqlitePool>) -> Html<String> {
    let user = ensure_auth(&cookies, &pool).await;
    let result =
        // TODO: https://github.com/launchbadge/sqlx/issues/1524
        sqlx::query_as::<_, SubmissionsItem>( "
select
  s.id as statement_id,
  s.text as statement_text,
  a.timestamp as author_timestamp,
  v.vote as vote,
  coalesce(sum(v_stats.vote == 1), 0) as yes_count,
  coalesce(sum(v_stats.vote == -1), 0) as no_count
from authors a
join statements s on s.id = a.statement_id
left outer join votes v on
  s.id = v.statement_id and a.user_id = v.user_id
left outer join votes v_stats on
  v_stats.statement_id = a.statement_id
where a.user_id = ?
group by a.statement_id
order by a.timestamp desc").bind(user.id).fetch_all(&pool).await.expect("Must be valid");

    let template = SubmissionsTemplate {
        submissions: result,
    };

    Html(template.render().unwrap())
}
