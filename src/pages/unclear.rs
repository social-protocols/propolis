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
use maud::{html, Markup};

use serde::Deserialize;
use sqlx::SqlitePool;
use tower_cookies::Cookies;

#[derive(Deserialize, Debug)]
pub struct UnclearForm {
    target_id: i64,
    typed_statement: String,
    referenced_statement_id: Option<i64>,
}

pub async fn unclear(
    Path(target_statement_id): Path<i64>,
    user: User,
    Extension(pool): Extension<SqlitePool>,
    base: BaseTemplate,
) -> Result<Markup, AppError> {
    let target_statement = get_statement(target_statement_id, &pool).await?;
    let user_opt = Some(user);

    let content = html! {
        div x-data="{ typed_statement: '', referenced_statement: null }" {
            div class="mb-8 rounded-lg shadow bg-white dark:bg-slate-700 flex" {
                (small_statement_content(&target_statement, None, false, &user_opt, &pool).await?)
                (small_statement_piechart(target_statement.id, &pool).await?)
                (small_statement_vote_fetch(target_statement.id, &user_opt, &pool).await?)
            }
            form method="post" hx-post={"/statement/"(target_statement_id)"/unclearrr"} hx-trigger="submit" hx-swap="none" {
                h2 class="text-xl mb-4" { "Unclear" }
                div class="mb-2" { "In which ways can this statement be interpreted? What was the original intention of the author?" }
                input type="hidden" name="target_id" value=(target_statement.id);

                template x-if="referenced_statement !== null" {
                    // preview of referenced statement
                    div {
                        input type="hidden" name="referenced_statement_id" x-model="referenced_statement.id";
                        div class="mb-2 flex" {
                            "Selected existing statement:"
                            button type="button" class="ml-auto px-4 py-1" x-on:click="referenced_statement = null" { "Cancel" };
                        }
                        div class="mb-5 p-4 rounded-lg shadow bg-white dark:bg-slate-700 flex" x-text="referenced_statement.text" {}
                    }
                }
                textarea
                    x-show="referenced_statement === null"
                    class="mb-4 dark:bg-slate-700 dark:text-white w-full p-4 border border-1 border-slate-500 dark:border-slate-200 rounded-lg"
                    rows = "4"
                    name="typed_statement"
                    // x-model="typed_statement" // TODO: x-model.fill https://github.com/lambda-fairy/maud/issues/240
                    placeholder="Careful, this is a new question to be understood independently. It's not a reply."
                    minlength="3"
                    hx-validate="true"
                    hx-target="#similar"
                    hx-post="/search"
                    hx-trigger="keyup changed delay:500ms, load"
                    {};

                div class="flex justify-end" {
                    button class="text-white bg-slate-500 px-4 py-1 rounded" { "submit" }
                }
            }
            div id="similar" {}
        }
    };

    Ok(base.title("Unclear").content(content).render())
}

pub async fn unclear_post(
    cookies: Cookies,
    Extension(pool): Extension<SqlitePool>,
    Form(form_data): Form<UnclearForm>,
) -> Result<Redirect, AppError> {
    // let user = User::get_or_create(&cookies, &pool).await?;
    // let target_segment = TargetSegment {
    //     statement_id: form_data.target_id,
    //     voted_yes: true,
    //     voted_no: true,
    // };

    println!("{form_data:?}");

    // let alternative_statement_id = match form_data.alternative_statement_id {
    //     Some(id) => id,
    //     None => {
    //         user.add_statement(form_data.typed_statement.as_str(), &pool)
    //             .await?
    //     }
    // };

    // add_alternative(form_data.target_id, alternative_statement_id, &pool).await?;
    // add_followup(target_segment, alternative_statement_id, &pool).await?;

    Ok(Redirect::to("/"))
}
