//! Various structs used all over

use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use serde::Deserialize;
use serde::Serialize;

use crate::error::Error;

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

#[derive(Debug, PartialEq, Deserialize, Copy, Clone, FromPrimitive)]
pub enum Vote {
    No = -1,
    Skip = 0,
    Yes = 1,
    ItDepends = 2,
}

impl Vote {
    pub fn from(vote: i64) -> Result<Vote, Error> {
        FromPrimitive::from_i64(vote).ok_or(Error::CustomError("Unknown vote value".to_string()))
    }
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

impl StatementStats {
    pub fn empty() -> Self {
        Self {
            yes_votes: 0,
            no_votes: 0,
            skip_votes: 0,
            itdepends_votes: 0,
            subscriptions: 0,
            total_votes: 0,
            participation: 0.0,
            polarization: 0.0,
            votes_per_subscription: 0.0,
        }
    }
}

pub struct TargetSegment {
    pub statement_id: i64,
    pub voted_yes: bool,
    pub voted_no: bool,
}
