//! One file per page

pub mod base;
pub mod charts;
pub mod index;
pub mod merge;
pub mod new_statement;
pub mod options;
#[cfg(feature="with_predictions")]
pub mod prediction;
pub mod statement;
pub mod statement_ui;
pub mod subscribe;
pub mod subscriptions;
pub mod vote;
