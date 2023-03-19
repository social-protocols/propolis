use super::base::base;
use crate::db::get_statement;
use crate::structs::{Statement, User};
use crate::util::base_url;
use crate::{db::autocomplete_statement, error::Error};

use axum::{extract::Query, response::Redirect, Extension, Form};
use http::HeaderMap;
use maud::{html, Markup};
use serde::Deserialize;
use sqlx::SqlitePool;
use tower_cookies::Cookies;

#[derive(Deserialize, Debug)]
pub struct NewStatementUrlQuery {
    target: Option<i64>,
}

fn html(target_statement: Option<Statement>) -> Markup {
    html! {
        form method="post" action="/create" {
            fieldset {
                legend { "Insert" }
                input
                    style="min-width: 80%"
                    name="statement_text"
                    _="on htmx:validation:validate
                      if my.value.length < 3
                        call me.setCustomValidity('Please enter a value')
                      else
                        call me.setCustomValidity('')
                      me.reportValidity()"
                    hx-validate="true"
                    hx-target="#similar"
                    hx-post="/completions"
                    hx-trigger="keyup changed delay:500ms";
                @if let Some(ref stmt) = target_statement {
                    input type="hidden" name="target" value=(stmt.id);
                }
                div {
                    button { "Add Statement" }
                }
            }
        }
        fieldset {
            legend { "Similar" }
            ul id="similar" {}
        }
        @if let Some(ref stmt) = target_statement {
            fieldset {
                legend { "Will be shown to people who answered or are following" }
                p {
                    a href=(format!("/statement/{}", stmt.id)) { (&stmt.text) }
                }
            }
        }
    }
}

pub async fn completions(
    header_map: HeaderMap,
    Extension(pool): Extension<SqlitePool>,
    Form(add_statement): Form<AddStatementForm>,
) -> Result<Markup, Error> {
    let statements = autocomplete_statement(add_statement.statement_text.as_str(), &pool).await?;
    Ok(html! {
        @for stmt in &statements {
            li {
                a href=(format!("{}/statement/{}", base_url(&header_map), stmt.id)) { (&stmt.text) }
            }
        }
    })
}

pub async fn new_statement(
    cookies: Cookies,
    maybe_user: Option<User>,
    url_query: Query<NewStatementUrlQuery>,
    Extension(pool): Extension<SqlitePool>,
) -> Result<Markup, Error> {
    let target_statement = match url_query.target {
        Some(target_id) => get_statement(target_id, &pool).await.ok().flatten(),
        None => None,
    };

    let content = html(target_statement);
    Ok(base(
        cookies,
        Some("New statement".to_string()),
        &maybe_user,
        content,
    )
    .into())
}

#[derive(Deserialize)]
pub struct AddStatementForm {
    statement_text: String,
    target: Option<i64>,
}

pub async fn create_statement(
    cookies: Cookies,
    Extension(pool): Extension<SqlitePool>,
    Form(form_data): Form<AddStatementForm>,
) -> Result<Redirect, Error> {
    let user = User::get_or_create(&cookies, &pool).await?;
    user.add_statement(form_data.statement_text, form_data.target, &pool)
        .await?;

    Ok(Redirect::to("/"))
}
