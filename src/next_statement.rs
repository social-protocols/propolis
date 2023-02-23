use sqlx::SqlitePool;

use crate::structs::Statement;

pub async fn next_statement_for_user(user_id: i64, pool: &SqlitePool) -> Option<Statement> {
    // try to pick a statement from the user's personal queue
    let statement = sqlx::query_as!(Statement,"select s.id as id, s.text as text from queue q join statements s on s.id = q.statement_id where q.user_id = ? limit 1", user_id)
            .fetch_optional(pool)
            .await
            .expect("Must be valid");

    // if there is no statement in the queue, pick a random statement
    match statement {
        Some(statement) => Some(statement),
        None => sqlx::query_as::<_, Statement>(
            // TODO: https://github.com/launchbadge/sqlx/issues/1524
            "SELECT id, text from statements ORDER BY RANDOM() LIMIT 1",
        )
        .fetch_optional(pool)
        .await
        .expect("Must be valid"),
    }
}

pub async fn next_statement_for_anonymous(pool: &SqlitePool) -> Option<Statement> {
    // for anonymous users, pick a random statement
    sqlx::query_as::<_, Statement>(
        // TODO: https://github.com/launchbadge/sqlx/issues/1524
        "SELECT id, text from statements ORDER BY RANDOM() LIMIT 1",
    )
    .fetch_optional(pool)
    .await
    .expect("Must be valid")
}
