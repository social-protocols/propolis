use super::base::base;
use crate::db::get_statement;
use crate::pages::statement_ui::small_statement_content;
use crate::structs::User;

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

    let content = html! {
        form method="post" action="/create" {
            h2 { "Create new statement" }
            div { "Make sure to add the full context, so that this statement can be understood alone." }
            textarea
                style="width: 100%"
                rows = "4"
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
                      hx-trigger="keyup changed delay:500ms" {};
            div style="display:flex; justify-content: flex-end;" {
                button { "Add Statement" }
            }
            @if let Some(ref statement) = target_statement {
                input type="hidden" name="target" value=(statement.id);
            }
            @if let Some(ref statement) = target_statement {
                h2 { "Replying to" }
                div style="margin-bottom: 5px" {"Your reply will be shown to people who subscribed or voted on this statement."}
                div.shadow style="display:flex; margin-bottom: 20px; border-radius: 10px;" {
                    (small_statement_content(&statement, None, &maybe_user, &pool).await?)
                }
            }
        }
        h2 { "Similar" }
        div id="similar" {}
    };
    Ok(base(
        cookies,
        Some("New statement".to_string()),
        &maybe_user,
        content,
    )
    .into())
}

pub async fn completions(
    _header_map: HeaderMap,
    maybe_user: Option<User>,
    Extension(pool): Extension<SqlitePool>,
    Form(add_statement): Form<AddStatementForm>,
) -> Result<Markup, Error> {
    let statements = autocomplete_statement(add_statement.statement_text.as_str(), &pool).await?;
    Ok(html! {
        @for statement in &statements {
            div.shadow style="display:flex; margin-bottom: 20px; border-radius: 10px;" {
                (small_statement_content(&statement, None, &maybe_user, &pool).await?)
            }
        }
    })
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
