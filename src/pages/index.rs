use crate::auth::ensure_auth;

use askama::Template;
use axum::{response::Html, Extension, Form};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tower_cookies::Cookies;

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
pub async fn index(cookies: Cookies, Extension(pool): Extension<SqlitePool>) -> Html<String> {
    let user = ensure_auth(&cookies, &pool).await;
    // try to pick a statement from the user's personal queue
    let statement =
        sqlx::query_as!(Statement,"select s.id as id, s.text as text from queue q join statements s on s.id = q.statement_id where q.user_id = ? limit 1", user.id)
            .fetch_optional(&pool)
            .await
            .expect("Must be valid");

    // if there is no statement in the queue, pick a random statement
    let statement: Option<Statement> = match statement {
        Some(statement) => Some(statement),
        None => sqlx::query_as::<_, Statement>(
            // TODO: https://github.com/launchbadge/sqlx/issues/1524
            "SELECT id, text from statements ORDER BY RANDOM() LIMIT 1",
        )
        .fetch_optional(&pool)
        .await
        .expect("Must be valid"),
    };

    let template = IndexTemplate {
        statement: &statement,
    };

    Html(template.render().unwrap())
}


#[derive(Deserialize)]
pub struct VoteForm {
    statement_id: i64,
    vote: i32,
}

pub async fn index_post(
    cookies: Cookies,
    Extension(pool): Extension<SqlitePool>,
    Form(vote): Form<VoteForm>,
) -> Html<String> {
    let user = ensure_auth(&cookies, &pool).await;

    sqlx::query!(
        "INSERT INTO votes (statement_id, user_id, vote) VALUES (?, ?, ?) on conflict ( statement_id, user_id) do update set vote = excluded.vote",
        vote.statement_id,
        user.id,
        vote.vote
    )
    .execute(&pool)
    .await
    .expect("Database problem");

    sqlx::query!(
        "INSERT INTO vote_history (user_id, statement_id, vote) VALUES (?, ?, ?)",
        user.id,
        vote.statement_id,
        vote.vote
    )
    .execute(&pool)
    .await
    .expect("Database problem");

    sqlx::query!(
        "delete from queue where user_id = ? and statement_id = ?",
        user.id,
        vote.statement_id
    )
    .execute(&pool)
    .await
    .expect("Database problem");

    index(cookies, Extension(pool)).await
}
