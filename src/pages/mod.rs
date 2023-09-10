//! One file per page

pub mod base_template;
pub mod charts;
pub mod frontpage;
pub mod new_statement;
#[cfg(feature = "with_predictions")]
pub mod prediction;
pub mod statement;
pub mod statement_ui;
pub mod subscribe;
pub mod subscriptions;
pub mod user;
pub mod vote;
