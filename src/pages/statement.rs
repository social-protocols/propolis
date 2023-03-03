use super::base::{get_base_template, BaseTemplate, GenericViewTemplate};
use crate::{db::get_statement, error::Error, structs::Statement};
use maud::{html, PreEscaped};

use askama::Template;
use axum::{extract::Path, response::Html, Extension};
use sqlx::SqlitePool;
use tower_cookies::Cookies;

#[derive(Template)]
#[template(path = "statement.j2")]
struct StatementTemplate<'a> {
    base: BaseTemplate,
    statement: &'a Option<Statement>,
}

/// Returns an apexchart div with votes of the particular statement
pub async fn votes(
    Path(statement_id): Path<i64>,
    cookies: Cookies,
    Extension(pool): Extension<SqlitePool>,
) -> Result<Html<String>, Error> {
    let statement = get_statement(statement_id, &pool).await?;

    if statement.is_none() {
        return Ok(Html("".to_string()))
    }

    let (a, s, d) = statement.unwrap().num_votes(&pool).await?;
    Ok(Html(html! {

        div id="chart" {}
        script type="text/javascript" {
            (format!("setupChart('#chart', {},{},{});", a, s, d))
        }
    }.into_string()))
}

fn html(statement: Option<Statement>) -> String {
    html! {
        @if let Some(statement) = statement {
            div.card.info {
                p { b { (statement.text) } }
                div.row {
                    div.col {
                        form id="form" hx-post="/vote" {
                            input type="hidden" value=(statement.id) name="statement_id" {
                                button name="vote" value="1" { "Agree" }
                                button name="vote" value="0" { "Skip" }
                                button name="vote" value="-1" { "Disagree" }
                                button
                                    name="showstats"
                                    hx-get=(format!("/votes/{}", statement.id))
                                    hx-target="#form"
                                    hx-swap="outerHTML"
                                {
                                    "Show stats"
                                }
                            }
                        }
                    }
                }
            }
        } @else {
            div {
                "Statement not found."
            }
        }
    }.into_string()
}

pub async fn statement(
    Path(statement_id): Path<i64>,
    cookies: Cookies,
    Extension(pool): Extension<SqlitePool>,
) -> Result<Html<String>, Error> {
    let statement = get_statement(statement_id, &pool).await?;

    let base = get_base_template(cookies, Extension(pool));
    let content = html(statement);
    GenericViewTemplate {
        base,
        content: content.as_str(),
        title: None,
    }
    .into()
}
