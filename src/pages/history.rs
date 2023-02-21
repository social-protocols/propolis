use crate::auth::ensure_auth;
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
    history: Vec<VoteHistoryItem>,
}

impl HistoryTemplate {
    fn human_relative_time(&self, timestamp: &i64) -> String {
        human_relative_time(timestamp)
    }
}

pub async fn history(cookies: Cookies, Extension(pool): Extension<SqlitePool>) -> Html<String> {
    let user = ensure_auth(&cookies, &pool).await;
    let query =
        sqlx::query_as!(VoteHistoryItem, "select s.id as statement_id, s.text as statement_text, timestamp as vote_timestamp, vote from vote_history v join statements s on s.id = v.statement_id where user_id = ? and vote != 0 order by timestamp desc", user.id);
    let result = query.fetch_all(&pool).await.expect("Must be valid");

    let template = HistoryTemplate { history: result };

    Html(template.render().unwrap())
}

