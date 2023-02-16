mod auth;

use auth::ensure_auth;
use axum::{
    response::{Html, Redirect},
    routing::{get, post},
    Extension, Form, Router,
};
use dotenvy::dotenv;
use std::env;
use tower_cookies::{CookieManagerLayer, Cookies};

use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use std::net::SocketAddr;

use askama::Template;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use timediff::*;

#[tokio::main]
async fn main() {
    dotenv().expect(".env file not found");
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .unwrap();

    let app = Router::new()
        .route("/", get(index))
        .route("/", post(index_post))
        .route("/new", get(new_statement))
        .route("/new", post(new_statement_post))
        .route("/history", get(history))
        .route("/submissions", get(submissions))
        .layer(Extension(pool))
        .layer(CookieManagerLayer::new());

    let addr = SocketAddr::from(([127, 0, 0, 1], 8000));
    println!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[derive(Serialize, sqlx::FromRow)]
struct Statement {
    id: i64,
    text: String,
}

#[derive(Template)]
#[template(path = "index.j2")]
struct IndexTemplate<'a> {
    statement: &'a Option<Statement>,
}

// Display one statement at random
async fn index(Extension(pool): Extension<SqlitePool>) -> Html<String> {
    let query =
        sqlx::query_as::<_, Statement>("SELECT id, text from statements ORDER BY RANDOM() LIMIT 1");
    let result = query.fetch_optional(&pool).await.expect("Must be valid");

    let template = IndexTemplate { statement: &result };

    Html(template.render().unwrap())
}

#[derive(Deserialize)]
struct VoteForm {
    statement_id: i64,
    vote: i32,
}

async fn index_post(
    cookies: Cookies,
    Extension(pool): Extension<SqlitePool>,
    Form(vote): Form<VoteForm>,
) -> Html<String> {
    let user = ensure_auth(&cookies, &pool).await;

    let query = sqlx::query!(
        "INSERT INTO votes (user_id, statement_id, vote) VALUES (?, ?, ?)",
        user.id,
        vote.statement_id,
        vote.vote
    )
    .execute(&pool)
    .await;
    query.expect("Database problem");

    index(Extension(pool)).await
}

#[derive(Template)]
#[template(path = "new_statement.j2")]
struct NewStatementTemplate {}

async fn new_statement() -> Html<String> {
    let template = NewStatementTemplate {};

    Html(template.render().unwrap())
}

#[derive(Deserialize)]
struct AddStatementForm {
    statement_text: String,
}

async fn new_statement_post(
    cookies: Cookies,
    Extension(pool): Extension<SqlitePool>,
    Form(add_statement): Form<AddStatementForm>,
) -> Redirect {
    let user = ensure_auth(&cookies, &pool).await;
    // TODO: add statement and author entry in transaction
    let created_statement = sqlx::query!(
        "INSERT INTO statements (text) VALUES (?) RETURNING id",
        add_statement.statement_text
    )
    .fetch_one(&pool)
    .await
    .expect("Database problem");

    let query = sqlx::query!(
        "INSERT INTO authors (user_id, statement_id) VALUES (?, ?)",
        user.id,
        created_statement.id
    )
    .execute(&pool)
    .await;
    query.expect("Database problem");

    // TODO: add statement to the user's queue, to vote on next

    Redirect::to("/")
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
        sqlx::query_as!(VoteHistoryItem, "select s.id as statement_id, s.text as statement_text, timestamp as vote_timestamp, vote from votes v join statements s on s.id = v.statement_id where user_id = ? and vote != 0 order by timestamp desc", user.id);
    let result = query.fetch_all(&pool).await.expect("Must be valid");

    let template = HistoryTemplate { history: result };

    Html(template.render().unwrap())
}

#[derive(sqlx::FromRow)]
struct SubmissionsItem {
    statement_id: i64,
    statement_text: String,
    author_timestamp: i64,
    vote_timestamp: i64, // nullable
    vote: i64,           // nullable
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
    let query =
        sqlx::query_as!(SubmissionsItem, "select s.id as statement_id, s.text as statement_text, a.timestamp as author_timestamp, v.timestamp as vote_timestamp, v.vote as vote from authors a join statements s on s.id = a.statement_id left outer join votes v on s.id = v.statement_id and a.user_id = v.user_id where a.user_id = ? order by a.timestamp desc", user.id);
    let result = query.fetch_all(&pool).await.expect("Must be valid");

    let template = SubmissionsTemplate {
        submissions: result,
    };

    Html(template.render().unwrap())
}
