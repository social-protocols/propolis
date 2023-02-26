//! Various structs used all over

use serde::Serialize;

/// Representation of a user. Provides various methods to find & update them
#[derive(Serialize, sqlx::FromRow, Debug)]
pub struct User {
    pub id: i64,
    pub secret: String,
}

/// Represents a voting history entry
#[derive(sqlx::FromRow)]
pub struct VoteHistoryItem {
    pub statement_id: i64,
    pub statement_text: String,
    pub vote_timestamp: i64,
    pub vote: i64,
}

/// Represents a statement
#[derive(Serialize, sqlx::FromRow)]
pub struct Statement {
    pub id: i64,
    pub text: String,
}
