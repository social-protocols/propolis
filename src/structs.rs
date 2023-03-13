//! Various structs used all over

use serde::Deserialize;
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

#[derive(PartialEq, Deserialize, Copy, Clone)]
#[non_exhaustive]
pub enum Vote {
    No = -1,
    Skip = 0,
    Yes = 1,
    ItDepends = 2,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct StatementStats {
    pub yes_votes: i64,
    pub no_votes: i64,
    pub skip_votes: i64,
    pub itdepends_votes: i64,
    pub subscriptions: i64,
    pub total_votes: i64,
    pub participation: f64,
    pub polarization: f64,
    pub votes_per_subscription: f64,
}
