use super::base::{get_base_template, GenericViewTemplate};
use crate::structs::User;
use crate::util::base_url;
use crate::{db::autocomplete_statement, error::Error};

use axum::{
    response::{Html, Redirect},
    Extension, Form,
};
use http::HeaderMap;
use maud::{html, PreEscaped};
use serde::Deserialize;
use sqlx::SqlitePool;
use tower_cookies::Cookies;

fn html() -> String {
    html! {
        form method="post" action="/new" {
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
                div {
                    button { "Add Statement" }
                }
            }
        }
        fieldset {
            legend { "Similar" }
            ul id="similar" {}
        }
    }
    .into_string()
}

pub async fn completions(
    header_map: HeaderMap,
    Extension(pool): Extension<SqlitePool>,
    Form(add_statement): Form<AddStatementForm>,
) -> Result<Html<String>, Error> {
    let statements = autocomplete_statement(add_statement.statement_text.as_str(), &pool).await?;
    Ok(Html(html! {
        @for stmt in &statements {
            li {
                a href=(format!("{}/statement/{}", base_url(&header_map), stmt.id)) { (PreEscaped(&stmt.text)) }
            }
        }
    }.into_string()))
}

pub async fn new_statement(
    cookies: Cookies,
    Extension(pool): Extension<SqlitePool>,
) -> Result<Html<String>, Error> {
    let base = get_base_template(cookies, Extension(pool));
    let content = html();
    GenericViewTemplate {
        base,
        content: content.as_str(),
        title: Some("New statement"),
    }
    .into()
}

#[derive(Deserialize)]
pub struct AddStatementForm {
    statement_text: String,
}

pub async fn new_statement_post(
    cookies: Cookies,
    Extension(pool): Extension<SqlitePool>,
    Form(add_statement): Form<AddStatementForm>,
) -> Result<Redirect, Error> {
    let user = User::get_or_create(&cookies, &pool).await?;
    user.add_statement(add_statement.statement_text, &pool)
        .await?;

    Ok(Redirect::to("/"))
}
