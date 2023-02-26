use super::base::{get_base_template, BaseTemplate};
use crate::{auth::User, error::Error};
use crate::util::human_relative_time;

use askama::Template;
use axum::{response::Html, Extension};
use sqlx::SqlitePool;
use tower_cookies::Cookies;

#[derive(sqlx::FromRow)]
pub struct VoteHistoryItem {
    statement_id: i64,
    statement_text: String,
    vote_timestamp: i64,
    vote: i64,
}

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
        Some(user) => {
            sqlx::query_as!(
                VoteHistoryItem,
                    "
select s.id as statement_id, s.text as statement_text, timestamp as vote_timestamp, vote from vote_history v
join statements s on
  s.id = v.statement_id
where user_id = ? and vote != 0
order by timestamp desc", user.id)
            .fetch_all(&pool).await?
        }
        None => Vec::new(),
    };

    let template = HistoryTemplate {
        base: get_base_template(cookies, Extension(pool)),
        history,
    };

    Ok(Html(template.render().unwrap()))
}
