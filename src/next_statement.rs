use crate::auth::User;

use axum::{response::Redirect, Extension};
use sqlx::SqlitePool;

pub async fn redirect_to_next_statement(
    existing_user: Option<User>,
    Extension(pool): Extension<SqlitePool>,
) -> Redirect {
    let statement_id: Option<i64> = match existing_user {
        Some(user) => next_statement_for_user(user.id, &pool).await,
        None => random_statement_id(&pool).await,
    };

    match statement_id {
        Some(id) => Redirect::to(format!("/statement/{}", id).as_str()),
        None => Redirect::to("/statement/0"), // TODO
    }
}

pub async fn next_statement_for_user(user_id: i64, pool: &SqlitePool) -> Option<i64> {
    // try to pick a statement from the user's personal queue
    let statement_id = next_statement_id_from_queue(user_id, pool).await;

    // if there is no statement in the queue, pick a random statement
    match statement_id {
        Some(statement_id) => Some(statement_id),
        None => random_statement_id(pool).await,
    }
}

pub async fn next_statement_id_from_queue(user_id: i64, pool: &SqlitePool) -> Option<i64> {
    sqlx::query_scalar!(
        "select statement_id from queue where user_id = ? limit 1",
        user_id
    )
    .fetch_optional(pool)
    .await
    .expect("Must be valid")
}

pub async fn random_statement_id(pool: &SqlitePool) -> Option<i64> {
    // for anonymous users, pick a random statement
    sqlx::query_scalar::<_, i64>(
        // TODO: https://github.com/launchbadge/sqlx/issues/1524
        "SELECT id from statements ORDER BY RANDOM() LIMIT 1",
    )
    .fetch_optional(pool)
    .await
    .expect("Must be valid")
}
