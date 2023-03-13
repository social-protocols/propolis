use super::base::base;
use crate::{db::get_statement, error::Error, structs::Statement};

use axum::{extract::Path, Extension};
use maud::{html, Markup};
use sqlx::SqlitePool;
use tower_cookies::Cookies;

/// Returns an apexchart div with votes of the particular statement
pub async fn votes(
    Path(statement_id): Path<i64>,
    Extension(pool): Extension<SqlitePool>,
) -> Result<Markup, Error> {
    let statement = get_statement(statement_id, &pool).await?;

    match statement {
        Some(statement) => {
            let (a, s, d) = statement.num_votes(&pool).await?;
            Ok(html! {
                div id="chart" {}
                script type="text/javascript" {
                    (format!("setupChart('#chart', {},{},{});", a, s, d))
                }
            })
        }
        None => Ok(html! {}),
    }
}

pub fn render_statement(statement: Statement) -> Markup {
    html! {
        div.statement {
            (statement.text)
        }
    }
}

pub async fn statement(
    Path(statement_id): Path<i64>,
    cookies: Cookies,
    Extension(pool): Extension<SqlitePool>,
) -> Result<Markup, Error> {
    let statement = get_statement(statement_id, &pool).await?;
    let content = html! {
        @if let Some(statement) = statement {
            (render_statement(statement))
            div.row {
                div.col {
                    form form id="form" hx-post="/vote" {
                        input type="hidden" value=(statement_id) name="statement_id";
                        button name="vote" value="Yes" { "Agree" }
                        button name="vote" value="Skip" { "Skip" }
                        button name="vote" value="ItDepends" { "It depends" }
                        button name="vote" value="No" { "Disagree" }
                    }
                }
            }
        } @else {
            div { "Statement not found." }
        }
    };

    Ok(base(cookies, None, content))
}
