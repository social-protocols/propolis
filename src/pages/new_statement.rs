use super::base::base;
use crate::db::{add_followup, get_statement};
use crate::pages::statement_ui::{
    small_statement_content, small_statement_piechart, small_statement_vote_fetch,
};
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

#[derive(Deserialize)]
pub struct AddStatementForm {
    statement_text: String,
    target: Option<i64>,
}

#[derive(Deserialize)]
pub struct LinkFollowupForm {
    statement_id: i64,
    followup_id: i64,
}

pub async fn new_statement(
    cookies: Cookies,
    maybe_user: Option<User>,
    url_query: Query<NewStatementUrlQuery>,
    Extension(pool): Extension<SqlitePool>,
) -> Result<Markup, Error> {
    let target_statement = match url_query.target {
        Some(target_id) => get_statement(target_id, &pool).await.ok(),
        None => None,
    };

    let content = html! {
        form method="post" action="/create" {
            h2 { "Create new statement" }
            div { "Ask if people agree with your statement. Make sure to add the full context, so that this statement can be understood alone." }
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
                h2 { "Following up on" }
                div style="margin-bottom: 5px" {"Your statement will be shown to people who subscribed or voted on this statement."}
                div.shadow style="display:flex; margin-bottom: 20px; border-radius: 10px;" {
                    (small_statement_content(&statement, None, &maybe_user, &pool).await?)
                    (small_statement_piechart(statement.id, &pool).await?)
                    (small_statement_vote_fetch(statement.id, &maybe_user, &pool).await?)
                }
            }
        }
        @if target_statement.is_some() {
            h2 { "Or link to an existing statement:" }
        } @else {
            h2 { "Similar statements:" }
        }
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
                @if let Some(target) = add_statement.target {
                    form method="post" action="/link_followup" {
                        input type="hidden" name="statement_id" value=(target);
                        input type="hidden" name="followup_id" value=(statement.id);
                        button { "Link" }
                    }
                }
                (small_statement_content(&statement, None, &maybe_user, &pool).await?)
                (small_statement_piechart(statement.id, &pool).await?)
                (small_statement_vote_fetch(statement.id, &maybe_user, &pool).await?)
            }
        }
    })
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

pub async fn link_followup(
    Extension(pool): Extension<SqlitePool>,
    Form(form_data): Form<LinkFollowupForm>,
) -> Result<Redirect, Error> {
    // TODO: is it ok that this linking can be done anonymously? Since no user record is needed for this query...
    add_followup(form_data.statement_id, form_data.followup_id, &pool).await?;

    Ok(Redirect::to("/"))
}
