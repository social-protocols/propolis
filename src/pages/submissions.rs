use super::base::{get_base_template, BaseTemplate};
use crate::db::get_submissions;
use crate::error::Error;
use crate::structs::User;
use crate::util::human_relative_time;

use askama::Template;
use axum::{response::Html, Extension};
use sqlx::SqlitePool;
use tower_cookies::Cookies;

#[derive(sqlx::FromRow)]
pub struct SubmissionsItem {
    statement_id: i64,
    statement_text: String,
    author_timestamp: i64,
    vote: i64, // vote is nullable, should be Option<i64>, but TODO: https://github.com/djc/askama/issues/752
    yes_count: i64,
    no_count: i64,
}

#[derive(Template)]
#[template(path = "submissions.j2")]
pub struct SubmissionsTemplate {
    base: BaseTemplate,
    submissions: Vec<SubmissionsItem>,
}

impl SubmissionsTemplate {
    fn human_relative_time(&self, timestamp: &i64) -> String {
        human_relative_time(timestamp)
    }
}

pub async fn submissions(
    maybe_user: Option<User>,
    cookies: Cookies,
    Extension(pool): Extension<SqlitePool>,
) -> Result<Html<String>, Error> {
    let submissions = match maybe_user {
        Some(user) => get_submissions(&user, &pool).await?,
        None => Vec::new(),
    };
    let template = SubmissionsTemplate {
        base: get_base_template(cookies, Extension(pool)),
        submissions,
    };

    Ok(Html(template.render().unwrap()))
}
