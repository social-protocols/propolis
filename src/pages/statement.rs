use crate::pages::base_template::BaseTemplate;
use crate::{
    db::{get_followups, get_statement},
    error::AppError,
    pages::statement_ui::{
        small_statement_content, small_statement_piechart, small_statement_vote,
        small_statement_vote_fetch,
    },
    structs::{PageMeta, Statement, User, Vote},
    util::base_url,
};

use axum::{extract::Path, Extension};
use http::HeaderMap;
use maud::{html, Markup};
use sqlx::SqlitePool;

use crate::db::random_statement_id;
use axum::response::Response;
use axum::response::{IntoResponse, Redirect};

use anyhow::Result;

pub async fn next_statement_id(
    existing_user: Option<User>,
    pool: &SqlitePool,
) -> Result<Option<i64>> {
    Ok(match existing_user {
        Some(user) => user.next_statement_for_user(pool).await?,
        None => random_statement_id(pool).await?,
    })
}

pub async fn statement_frontpage(
    existing_user: Option<User>,
    maybe_user: Option<User>,
    Extension(pool): Extension<SqlitePool>,
    base: BaseTemplate,
) -> Result<Response, AppError> {
    let statement_id = next_statement_id(existing_user, &pool).await?;

    Ok(match statement_id {
        Some(id) => Redirect::to(format!("/statement/{id}").as_str()).into_response(),
        None => base
            .content(history(&maybe_user, &pool).await?)
            .render()
            .into_response(),
    })
}

pub async fn statement_page(
    Path(statement_id): Path<i64>,
    maybe_user: Option<User>,
    Extension(pool): Extension<SqlitePool>,
    headers: HeaderMap,
    base: BaseTemplate,
) -> Result<Markup, AppError> {
    let statement: Option<Statement> = get_statement(statement_id, &pool).await.ok();
    let user_vote = match &maybe_user {
        Some(user) => user.get_vote(statement_id, &pool).await?,
        None => None,
    };
    let content = html! {
        @if let Some(statement) = &statement {
            div data-testid="current-statement" class="rounded-lg shadow bg-white dark:bg-slate-700 flex " {
                div data-testid="statement-text" class="w-full text-xl p-6" {
                    (statement.text)
                }
                @if user_vote.is_some() {
                    (small_statement_piechart(statement.id, &pool).await?)
                    (small_statement_vote(user_vote)?)
                }
            }
            form hx-post="/vote" {
                input type="hidden" value=(statement_id) name="statement_id";
                div class="flex gap-2 mb-12 mt-3" {
                    button class="text-white bg-green-600 px-4 py-1 rounded" name="vote" value="Yes" { "YES" }
                    button class="text-white bg-red-600 px-4 py-1 rounded" name="vote" value="No" { "NO" }
                    button class="px-4 py-1" name="vote" value="Skip" { "skip / I don't know" }
                }
            }
            @match user_vote {
                Some(_) => {
                    h2 class="text-xl mb-4" { "Follow-ups" }
                    @let followups = get_followups(statement_id, &pool).await?;
                    @if followups.is_empty() {
                        div { "No follow-ups yet." }
                    }
                    @for statement_id in followups {
                        // TODO: different columns depending on vote-dependent follow up
                        div class="mb-5 rounded-lg shadow bg-white dark:bg-slate-700 flex" {
                            (small_statement_content(&get_statement(statement_id, &pool).await?, None, true, &maybe_user, &pool).await?)
                            (small_statement_piechart(statement_id, &pool).await?)
                            (small_statement_vote_fetch(statement_id, &maybe_user, &pool).await?)
                        }
                    }
                }
                None => (history(&maybe_user, &pool).await?)
            }
        } @else {
            div { "Question not found." }
        }
    };

    let page_meta = statement.as_ref().map(|statement| PageMeta {
        title: Some("Yes or no?".to_string()),
        description: Some(statement.text.to_owned()),
        url: Some(format!("{}/statement/{}", base_url(&headers), statement_id)),
    });

    Ok(base.content(content).page_meta_opt(page_meta).into())
}

pub async fn history(maybe_user: &Option<User>, pool: &SqlitePool) -> Result<Markup, AppError> {
    let history_items = match maybe_user {
        Some(user) => user.vote_history(20, pool).await?,
        None => Vec::new(),
    };

    Ok(html! {
        @if !history_items.is_empty() {
            h2 class="text-xl mb-4" { "Recent votes" }
        }
        @for item in history_items {
            @let statement = Statement {
                id: item.statement_id,
                text: item.statement_text,
            };
            div class="mb-5 rounded-lg shadow bg-white dark:bg-slate-700 flex" {
                (small_statement_content(&statement, None, true, maybe_user, pool).await?)
                (small_statement_piechart(item.statement_id, pool).await?)
                (small_statement_vote(Some(Vote::from(item.vote)?))?)
            }
        }
    })
}
