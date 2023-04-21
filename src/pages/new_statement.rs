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
}

#[derive(Deserialize)]
pub struct AddStatementForm {
    statement_text: String,
    target_id: Option<i64>,
    target_yes: Option<bool>,
    target_no: Option<bool>,
}

#[derive(Deserialize)]
pub struct LinkFollowupForm {
    statement_id: i64,
    followup_id: i64,
    target_yes: bool,
    target_no: bool,
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
                            input type="checkbox" name="target_yes" id="target_yes"  value="true" checked[url_query.target_yes == Some(true)];
                            "Yes"
                        }
                        label for="target_no" {
                            input type="checkbox" name="target_no" id="target_no" value="true" checked[url_query.target_no == Some(true)];
                            "No"
                        }
                    }
                    div style="margin-bottom: 5px" {"Your statement will be shown to people who subscribed to or voted on:"}
                    div.shadow style="display:flex; margin-bottom: 20px; border-radius: 10px;" {
                        (small_statement_content(statement, None, false, &maybe_user, &pool).await?)
                        (small_statement_piechart(statement.id, &pool).await?)
                    }
                }
                div { "Ask if people agree with your statement. Make sure to add the full context, so that this statement can be understood alone." }
                textarea
                    style="width: 100%"
                    rows = "4"
                    name="statement_text"
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
            @if target_statement.is_some() {
                h2 { "Or link to an existing statement:" }
            } @else {
                h2 { "Similar statements:" }
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
    let statements = search_statement(form.statement_text.as_str(), &pool).await?;
    Ok(html! {
        @for search_result_statement in &statements {
            div.shadow style="display:flex; margin-bottom: 20px; border-radius: 10px;" {
                @if let Some(target) = form.target_id {
                    form method="post" action="/link_followup" {
                        input type="hidden" name="statement_id" value=(target);
                        input type="hidden" name="target_yes" value=(form.target_yes.unwrap_or(false));
                        input type="hidden" name="target_no" value=(form.target_no.unwrap_or(false));
                        input type="hidden" name="followup_id" value=(search_result_statement.id);
                        button { "Link" }
                    }
                }
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
    user.add_statement(form_data.statement_text, target_segment, &pool)
        .await?;

    Ok(Redirect::to("/"))
}

pub async fn link_followup(
    Extension(pool): Extension<SqlitePool>,
    Form(form_data): Form<LinkFollowupForm>,
) -> Result<Redirect, AppError> {
    // TODO: is it ok that this linking can be done anonymously? Since no user record is needed for this query...
    add_followup(
        TargetSegment {
            statement_id: form_data.statement_id,
            voted_yes: form_data.target_yes,
            voted_no: form_data.target_no,
        },
        form_data.followup_id,
        &pool,
    )
    .await?;

    Ok(Redirect::to("/"))
}
