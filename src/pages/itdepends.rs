use super::base::BaseTemplate;
use crate::db::{add_alternative, add_followup, get_statement, search_statement};
use crate::pages::statement_ui::{
    small_statement_content, small_statement_piechart, small_statement_vote_fetch,
};
use crate::structs::{TargetSegment, User};

use crate::error::AppError;

use axum::extract::Path;
use axum::response::Redirect;
use axum::{Extension, Form};
use http::HeaderMap;
use maud::{html, Markup};

use serde::Deserialize;
use sqlx::SqlitePool;
use tower_cookies::Cookies;

#[derive(Deserialize)]
pub struct ItDependsForm {
    typed_statement: String,
    alternative_statement_id: Option<i64>,
    target_id: i64,
}

pub async fn itdepends(
    Path(target_statement_id): Path<i64>,
    user: User,
    Extension(pool): Extension<SqlitePool>,
    base: BaseTemplate,
) -> Result<Markup, AppError> {
    let target_statement = get_statement(target_statement_id, &pool).await?;
    let user_opt = Some(user);

    let content = html! {
        div x-data="{ typed_statement: '', alternative_statement: null }" {
            div class="mb-8 rounded-lg shadow bg-white dark:bg-slate-700 flex" {
                (small_statement_content(&target_statement, None, false, &user_opt, &pool).await?)
                (small_statement_piechart(target_statement.id, &pool).await?)
                (small_statement_vote_fetch(target_statement.id, &user_opt, &pool).await?)
            }
            h2 class="text-xl mb-4" { "It Depends" }
            div class="mb-2" { "Provide an alternative statement which clarifies context and/or definitions." }
            form method="post" action={"/statement/"(target_statement_id)"/itdepends"} {
                input type="hidden" name="target_id" value=(target_statement.id);
                // preview of selected existing alternative statement
                template x-if="alternative_statement !== null" {
                    div {
                        input type="hidden" name="alternative_statement_id" x-model="alternative_statement.id";
                        div class="mb-2 flex" {
                            "Selected existing statement:"
                            button type="button" class="ml-auto px-4 py-1" x-on:click="alternative_statement = null" { "Cancel" };
                        }
                        div class="mb-5 p-4 rounded-lg shadow bg-white dark:bg-slate-700 flex" x-text="alternative_statement.text" {}
                    }
                }
                textarea
                    x-show="alternative_statement === null"
                    class="mb-4 dark:bg-slate-700 dark:text-white w-full p-4 border border-1 border-slate-500 dark:border-slate-200 rounded"
                    rows = "4"
                    name="typed_statement"
                    x-model="typed_statement" // TODO: x-model.fill https://github.com/lambda-fairy/maud/issues/240
                    placeholder="Careful, this is a new statement to be understood independently. It's not a reply."
                    minlength="3"
                    hx-validate="true"
                    hx-target="#similar"
                    hx-post="/itdepends_completions"
                    hx-trigger="keyup changed delay:500ms, load"
                    {};
                // template x-if="alternative_statement === null" {
                //     div x-show="typed_statement.length > 0" {
                //         div class="mb-2" { "Preview:" }
                //         div class="mb-4 p-4 rounded-lg shadow bg-white dark:bg-slate-700 flex" x-text="typed_statement" {}
                //     }
                // }
                div class="flex justify-end" {
                    button class="text-white bg-slate-500 px-4 py-1 rounded" { "Propose Alternative Statement" }
                }
            }
            div id="similar" {}
        }
    };

    Ok(base.title("It Depends").content(content).render())
}

pub async fn itdepends_create(
    cookies: Cookies,
    Extension(pool): Extension<SqlitePool>,
    Form(form_data): Form<ItDependsForm>,
) -> Result<Redirect, AppError> {
    let user = User::get_or_create(&cookies, &pool).await?;
    let target_segment = TargetSegment {
        statement_id: form_data.target_id,
        voted_yes: true,
        voted_no: true,
    };

    let alternative_statement_id = match form_data.alternative_statement_id {
        Some(id) => id,
        None => user.add_statement(form_data.typed_statement, &pool).await?,
    };

    add_alternative(form_data.target_id, alternative_statement_id, &pool).await?;
    add_followup(target_segment, alternative_statement_id, &pool).await?;

    Ok(Redirect::to("/"))
}

pub async fn itdepends_completions(
    _header_map: HeaderMap,
    maybe_user: Option<User>,
    Extension(pool): Extension<SqlitePool>,
    Form(form): Form<ItDependsForm>,
) -> Result<Markup, AppError> {
    let statements = search_statement(form.typed_statement.as_str(), &pool).await?;
    Ok(html! {
        @if !statements.is_empty() {
            h2 class="text-xl mb-4" { "Did you mean" }
        }
        @for search_result_statement in &statements {
            div class="mb-5 rounded-lg shadow bg-white dark:bg-slate-700 flex" {
                button class="text-white bg-slate-500 px-4 py-1 rounded" x-on:click={"alternative_statement = {'id': "(search_result_statement.id)", 'text': '"(search_result_statement.text_original.replace('\'', "\\'"))"'}"} { "Use" }
                (small_statement_content(&search_result_statement.statement_highlighted(), None, true, &maybe_user, &pool).await?)
                (small_statement_piechart(search_result_statement.id, &pool).await?)
                (small_statement_vote_fetch(search_result_statement.id, &maybe_user, &pool).await?)
            }
        }
    })
}
