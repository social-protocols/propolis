use super::base::base;
use crate::db::{add_alternative, get_statement, search_statement};
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
    cookies: Cookies,
    user: User,
    Extension(pool): Extension<SqlitePool>,
    headers: HeaderMap,
) -> Result<Markup, AppError> {
    let target_statement = get_statement(target_statement_id, &pool).await?;
    let user_opt = Some(user);

    let content = html! {
        div x-data="{ typed_statement: '', alternative_statement: null }" {
            div.shadow style="display:flex; margin-bottom: 20px; border-radius: 10px;" {
                (small_statement_content(&target_statement, None, false, &user_opt, &pool).await?)
                (small_statement_piechart(target_statement.id, &pool).await?)
                (small_statement_vote_fetch(target_statement.id, &user_opt, &pool).await?)
            }
            h2 { "It Depends" }
            div { "Provide an alternative statement which clarifies context and/or definitions." }
            form method="post" action={"/statement/"(target_statement_id)"/itdepends"} {
                input type="hidden" name="target_id" value=(target_statement.id);
                // preview of selected existing alternative statement
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
                    _="on htmx:validation:validate
                          if my.value.length < 3
                            call me.setCustomValidity('Please enter a value')
                          else
                            call me.setCustomValidity('')
                          me.reportValidity()"
                          hx-validate="true"
                          hx-target="#similar"
                          hx-post="/itdepends_completions"
                          hx-trigger="keyup changed delay:500ms, load" {};
                template x-if="alternative_statement === null" {
                    div x-show="typed_statement.length > 0" {
                        div {
                            "Preview:"
                        }
                        div.shadow style="display:flex; margin-bottom: 20px; border-radius: 10px; padding: 15px;" x-text="typed_statement" {}
                    }
                }
                div style="display:flex; justify-content: flex-end;" {
                    button { "Propose Alternative Statement" }
                }
            }
            h2 { "Did you mean" }
            div id="similar" {}
        }
    };

    Ok(base(
        cookies,
        Some("It Depends".to_string()),
        &user_opt,
        content,
        &headers,
        None,
    ))
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
        None => {
            user.add_statement(form_data.typed_statement, Some(target_segment), &pool)
                .await?
        }
    };

    add_alternative(form_data.target_id, alternative_statement_id, &pool).await?;

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
