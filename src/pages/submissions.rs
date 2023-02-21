use crate::auth::ensure_auth;
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
    submissions: Vec<SubmissionsItem>,
}

impl SubmissionsTemplate {
    fn human_relative_time(&self, timestamp: &i64) -> String {
        human_relative_time(timestamp)
    }
}

pub async fn submissions(cookies: Cookies, Extension(pool): Extension<SqlitePool>) -> Html<String> {
    let user = ensure_auth(&cookies, &pool).await;
    let result =
        // TODO: https://github.com/launchbadge/sqlx/issues/1524
        sqlx::query_as::<_, SubmissionsItem>( "
select
  s.id as statement_id,
  s.text as statement_text,
  a.timestamp as author_timestamp,
  v.vote as vote,
  coalesce(sum(v_stats.vote == 1), 0) as yes_count,
  coalesce(sum(v_stats.vote == -1), 0) as no_count
from authors a
join statements s on s.id = a.statement_id
left outer join votes v on
  s.id = v.statement_id and a.user_id = v.user_id
left outer join votes v_stats on
  v_stats.statement_id = a.statement_id
where a.user_id = ?
group by a.statement_id
order by a.timestamp desc").bind(user.id).fetch_all(&pool).await.expect("Must be valid");

    let template = SubmissionsTemplate {
        submissions: result,
    };

    Html(template.render().unwrap())
}
