//! Various structs used all over

use anyhow::anyhow;
use anyhow::Result;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use serde::Deserialize;
use serde::Serialize;

/// Representation of a user. Provides various methods to find & update them
#[derive(Serialize, sqlx::FromRow, Debug, Clone)]
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
#[derive(Serialize, sqlx::FromRow, Clone, Debug)]
pub struct Statement {
    pub id: i64,
    pub text: String,
}

#[derive(Serialize, sqlx::FromRow, Clone)]
pub struct SearchResultStatement {
    pub id: i64,
    pub text_highlighted: String,
    pub text_original: String,
}

impl SearchResultStatement {
    pub fn statement_highlighted(&self) -> Statement {
        Statement {
            id: self.id,
            text: self.text_highlighted.clone(),
        }
    }
}

#[derive(Debug, PartialEq, Deserialize, Copy, Clone, FromPrimitive)]
pub enum Vote {
    No = -1,
    Skip = 0,
    Yes = 1,
}

impl Vote {
    pub fn from(vote: i64) -> Result<Vote> {
        FromPrimitive::from_i64(vote).ok_or(anyhow!("Unknown vote value: {}", vote))
    }

    /// Returns a factor that you can use in formulas. Maps ItDepends to 0.
    pub fn to_factor(self) -> i64 {
        match self {
            Vote::No => -1,
            Vote::Skip => 0,
            Vote::Yes => 1,
            Vote::ItDepends => 0,
        }
    }
}

#[derive(Serialize, sqlx::FromRow)]
pub struct StatementStats {
    pub yes_votes: i64,
    pub no_votes: i64,
    pub skip_votes: i64,
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

#[derive(Clone)]
pub struct PageMeta {
    pub title: Option<String>,
    pub description: Option<String>,
    pub url: Option<String>,
}

#[derive(Serialize, sqlx::FromRow, Clone)]
pub struct StatementPrediction {
    pub statement_id: i64,
    pub ai_env: String,
    pub prompt_name: String,
    pub prompt_version: i64,
    pub prompt_result: String,
    pub completion_tokens: i64,
    pub prompt_tokens: i64,
    pub total_tokens: i64,
    pub created: i64,
}

impl From<StatementPrediction> for String {
    fn from(value: StatementPrediction) -> Self {
        serde_json::to_string_pretty(&value).unwrap_or("<serde_json failure>".to_string())
    }
}
