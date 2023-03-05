use super::base::{get_base_template, GenericViewTemplate};
use crate::{db::get_statement, error::Error};

use axum::{extract::Path, response::Html, Extension};
use maud::{html, PreEscaped};
use sqlx::SqlitePool;
use tower_cookies::Cookies;

pub async fn statement(
    Path(statement_id): Path<i64>,
    cookies: Cookies,
    Extension(pool): Extension<SqlitePool>,
) -> Result<Html<String>, Error> {
    let statement = get_statement(statement_id, &pool).await?;

    let content = html! {
        @if let Some(statement) = statement {
            div.card.info {
                p {
                    b { (PreEscaped(&statement.text)) }
                }
                div.row {
                    div.col {
                        form method="post" action="/vote" {
                            input type="hidden" value=(statement.id) name="statement_id";
                            button name="vote" value="Yes" { "Agree" }
                            button name="vote" value="Skip" { "Skip" }
                            button name="vote" value="ItDepends" { "It depends" }
                            button name="vote" value="No" { "Disagree" }
                        }
                    }
                }
            }
        } @else {
            div { "Statement not found." }
        }
    };

    let base = get_base_template(cookies, Extension(pool));
    GenericViewTemplate {
        base,
        content: content.into_string().as_str(),
        title: None,
    }
    .into()
}
