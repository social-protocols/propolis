use super::base::base;
use crate::db::{add_followup, get_statement};
use crate::error::AppError;
use crate::pages::statement_ui::{
    small_statement_content, small_statement_piechart, small_statement_vote_fetch,
};
use crate::structs::{TargetSegment, User};

use crate::db::search_statement;

use axum::{extract::Query, response::Redirect, Extension, Form};
use http::HeaderMap;
use maud::{html, Markup};
use serde::Deserialize;
use sqlx::SqlitePool;
use tower_cookies::Cookies;

#[derive(Deserialize, Debug)]
pub struct NewStatementUrlQuery {
    target: Option<i64>,
    target_yes: Option<bool>,
    target_no: Option<bool>,
    target_all: Option<bool>,
}

#[derive(Deserialize)]
pub struct AddStatementForm {
    typed_statement: String,
    alternative_statement_id: Option<i64>,
    target_id: Option<i64>,
    target_yes: Option<bool>,
    target_no: Option<bool>,
}

pub async fn new_statement(
    cookies: Cookies,
    maybe_user: Option<User>,
    url_query: Query<NewStatementUrlQuery>,
    Extension(pool): Extension<SqlitePool>,
    headers: HeaderMap,
) -> Result<Markup, AppError> {
    let target_statement = match url_query.target {
        Some(target_id) => get_statement(target_id, &pool).await.ok(),
        None => None,
    };

    let content = html! {
        div x-data="{ typed_statement: '', alternative_statement: null }" {
            form method="post" action="/create" {
                h2 { "Create new statement" }
                @if let Some(ref statement) = target_statement {
                    input type="hidden" name="target_id" value=(statement.id);
                    div {
                        "Target people who voted:"
                        label style="padding-left: 20px; padding-right: 20px" for="target_yes" {
                            input type="checkbox" name="target_yes" id="target_yes"  value="true" checked[url_query.target_yes == Some(true) || url_query.target_all == Some(true)];
                            "Yes"
                        }
                        label for="target_no" {
                            input type="checkbox" name="target_no" id="target_no" value="true" checked[url_query.target_no == Some(true) || url_query.target_all == Some(true)];
                            "No"
                        }
                    }
                    div.shadow style="display:flex; margin-bottom: 20px; border-radius: 10px;" {
                        (small_statement_content(statement, None, false, &maybe_user, &pool).await?)
                        (small_statement_piechart(statement.id, &pool).await?)
                    }
                }
                div { "Ask if people agree with your statement. Make sure to add the full context, so that this statement can be understood alone." }
                template x-if="alternative_statement !== null" {
                    div {
                        input type="hidden" name="alternative_statement_id" x-model="alternative_statement.id";
                        div style="display:flex;" {
                            "Selected existing statement:"
                            button type="button" style="margin-left:auto;" x-on:click="alternative_statement = null" { "Cancel" };
                        }
                        div.shadow style="display:flex; margin-bottom: 20px; border-radius: 10px; padding: 15px;" x-text="alternative_statement.text" {}
                    }
                }
                textarea
                    x-show="alternative_statement === null"
                    style="width: 100%"
                    rows = "4"
                    name="typed_statement"
                    x-model="typed_statement" // TODO: x-model.fill https://github.com/lambda-fairy/maud/issues/240
                    placeholder="Careful, this is a new statement to be understood independently. It's not a reply."
                    minLength="3"
                    hx-validate="true"
                    hx-target="#similar"
                    hx-post="/new/completions"
                    hx-trigger="keyup changed delay:500ms, load"
                    data-testid="create-statement-field"
                    {};
                template x-if="alternative_statement === null" {
                    div x-show="typed_statement.length > 0" {
                        div {
                            "Preview:"
                        }
                        div.shadow style="display:flex; margin-bottom: 20px; border-radius: 10px; padding: 15px;" x-text="typed_statement" {}
                    }
                }
                div style="display:flex; justify-content: flex-end;" {
                    button data-testid="create-statement-submit" { "Add Statement" }
                }
            }
            div id="similar" {}
        }
    };
    Ok(base(
        cookies,
        Some("New statement".to_string()),
        &maybe_user,
        content,
        &headers,
        None,
    ))
}

pub async fn new_statement_completions(
    _header_map: HeaderMap,
    maybe_user: Option<User>,
    Extension(pool): Extension<SqlitePool>,
    Form(form): Form<AddStatementForm>,
) -> Result<Markup, AppError> {
    let statements = search_statement(form.typed_statement.as_str(), &pool).await?;
    Ok(html! {
        @if !statements.is_empty() {
            h2 { "Did you mean" }
        }
        @for search_result_statement in &statements {
            div.shadow style="display:flex; margin-bottom: 20px; border-radius: 10px;" {
                button x-on:click={"alternative_statement = {'id': "(search_result_statement.id)", 'text': '"(search_result_statement.text_original.replace('\'', "\\'"))"'}"} { "Use" }
                (small_statement_content(&search_result_statement.statement_highlighted(), None, true, &maybe_user, &pool).await?)
                (small_statement_piechart(search_result_statement.id, &pool).await?)
                (small_statement_vote_fetch(search_result_statement.id, &maybe_user, &pool).await?)
            }
        }
    })
}

pub async fn create_statement(
    cookies: Cookies,
    Extension(pool): Extension<SqlitePool>,
    Form(form_data): Form<AddStatementForm>,
) -> Result<Redirect, AppError> {
    let user = User::get_or_create(&cookies, &pool).await?;
    let target_segment = match form_data.target_id {
        Some(target_id) => Some(TargetSegment {
            statement_id: target_id,
            voted_yes: form_data.target_yes.unwrap_or(false),
            voted_no: form_data.target_no.unwrap_or(false),
        }),
        None => None,
    };

    let statement_id = match form_data.alternative_statement_id {
        Some(id) => id,
        None => user.add_statement(form_data.typed_statement, &pool).await?,
    };

    if let Some(target_segment) = target_segment {
        add_followup(target_segment, statement_id, &pool).await?;
    }

    Ok(Redirect::to("/"))
}
