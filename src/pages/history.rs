use super::base::{get_base_template, BaseTemplate};
use crate::db::VoteHistoryItem;
use crate::util::human_relative_time;
use crate::{auth::User, error::Error};

use askama::Template;
use axum::{response::Html, Extension};
use sqlx::SqlitePool;
use tower_cookies::Cookies;

#[derive(Template)]
#[template(path = "history.j2")]
pub struct HistoryTemplate {
    base: BaseTemplate,
    history: Vec<VoteHistoryItem>,
}

impl HistoryTemplate {
    fn human_relative_time(&self, timestamp: &i64) -> String {
        human_relative_time(timestamp)
    }
}

pub async fn history(
    maybe_user: Option<User>,
    cookies: Cookies,
    Extension(pool): Extension<SqlitePool>,
) -> Result<Html<String>, Error> {
    let history = match maybe_user {
        Some(user) => user.vote_history(&pool).await?,
        None => Vec::new(),
    };

    let template = HistoryTemplate {
        base: get_base_template(cookies, Extension(pool)),
        history,
    };

    Ok(Html(template.render().unwrap()))
}
