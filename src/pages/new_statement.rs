use super::base_template::BaseTemplate;
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
    maybe_user: Option<User>,
    url_query: Query<NewStatementUrlQuery>,
    Extension(pool): Extension<SqlitePool>,
    base: BaseTemplate,
) -> Result<Markup, AppError> {
    let target_statement = match url_query.target {
        Some(target_id) => get_statement(target_id, &pool).await.ok(),
        None => None,
    };

    let content = html! {
        div x-data="{ typed_statement: '', alternative_statement: null }" {
            form method="post" action="/create" {
                h2 class="text-xl mb-4" { "Ask Question" }
                div { "A good question..." }
                ul class="mb-2 list-disc list-inside" {
                    li { "can only be answered with YES or NO" }
                    li { "can be understood without additional context" }
                }
                template x-if="alternative_statement !== null" {
                    div {
                        input type="hidden" name="alternative_statement_id" x-model="alternative_statement.id";
                        div class="mb-2 flex" {
                            "Selected existing question:"
                            button type="button" class="ml-auto px-4 py-1" x-on:click="alternative_statement = null" { "Cancel" };
                        }
                        div class="mb-5 p-4 rounded-lg shadow bg-white dark:bg-slate-700 flex" x-text="alternative_statement.text" {}
                    }
                }
                textarea
                    x-show="alternative_statement === null"
                    class="mb-4 dark:bg-slate-700 dark:text-white w-full p-4 border border-1 border-slate-500 dark:border-slate-200 rounded-lg"
                    rows = "4"
                    name="typed_statement"
                    x-model="typed_statement" // TODO: x-model.fill https://github.com/lambda-fairy/maud/issues/240
                    placeholder="Is climate change caused by human activities?"
                    required
                    minLength="3"
                    hx-validate="true"
                    hx-target="#similar"
                    hx-post="/new/completions"
                    hx-trigger="keyup changed delay:500ms, load"
                    data-testid="create-statement-field"
                    {};
                // template x-if="alternative_statement === null" {
                //     div x-show="typed_statement.length > 0" {
                        // div class="mb-2" { "Preview:" }
                //         div class="mb-4 p-4 rounded-lg shadow bg-white dark:bg-slate-700" x-text="typed_statement" {}

                //         // div class="flex gap-2 mb-12 mt-3" {
                //         //     button class="border-dashed border-2 border-green-600 px-4 py-1 rounded" name="vote" value="Yes" { "YES" }
                //         //     button class="border-dashed border-2 border-red-600 px-4 py-1 rounded" name="vote" value="No" { "NO" }
                //         // }
                //     }
                // }
                @if let Some(ref statement) = target_statement {
                    input type="hidden" name="target_id" value=(statement.id);
                    div class="mb-4" {
                        "Shown to people who voted:"
                        label class="px-5" for="target_yes" {
                            input type="checkbox" name="target_yes" id="target_yes"  value="true" checked[url_query.target_yes == Some(true) || url_query.target_all == Some(true)];
                            span class="ml-1" { "Yes" }
                        }
                        label for="target_no" {
                            input type="checkbox" name="target_no" id="target_no" value="true" checked[url_query.target_no == Some(true) || url_query.target_all == Some(true)];
                            span class="ml-1" { "No" }
                        }
                    }
                    div class="mb-5 rounded-lg shadow bg-white dark:bg-slate-700 flex" {
                        (small_statement_content(statement, None, false, &maybe_user, &pool).await?)
                        (small_statement_piechart(statement.id, &pool).await?)
                    }
                }
                div class="flex justify-end" {
                    button data-testid="create-statement-submit" class="text-white bg-slate-500 px-4 py-1 rounded" { "Submit" }
                }
            }
            div id="similar" {}
        }
    };
    Ok(base.title("Ask Question").content(content).into())
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
            h2 class="text-xl mb-4" { "Did you mean" }
        }
        @for search_result_statement in &statements {
            div class="mb-5 rounded-lg shadow bg-white dark:bg-slate-700 flex" {
                button
                    class="text-white bg-slate-500 px-4 py-1 rounded"
                    x-on:click={"alternative_statement = {'id': "(search_result_statement.id)", 'text': '"(search_result_statement.text_original.replace('\'', "\\'"))"'}"}
                    { "Use" }
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
        None => {
            user.add_statement(form_data.typed_statement.as_str(), &pool)
                .await?
        }
    };

    if let Some(target_segment) = target_segment {
        add_followup(target_segment, statement_id, &pool).await?;
    }

    Ok(Redirect::to(&format!("/statement/{statement_id}")))
}
