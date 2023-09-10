//! One file per page

pub mod base_template;
pub mod charts;
pub mod frontpage;
pub mod merge;
pub mod new_statement;
pub mod options;
#[cfg(feature = "with_predictions")]
pub mod prediction;
pub mod statement;
pub mod statement_ui;
pub mod subscribe;
pub mod subscriptions;
pub mod user;
pub mod vote;
