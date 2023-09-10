use crate::pages::statement_ui::{
    inline_statement_content, inline_statement_piechart, inline_statement_vote_fetch,
};
use crate::structs::User;
use crate::{db, error::AppError};

use anyhow::Result;
use axum::{Extension, Form};
use maud::{html, Markup};
use serde::Deserialize;
use sqlx::SqlitePool;

use crate::pages::base_template::BaseTemplate;

pub async fn frontpage(
    maybe_user: Option<User>,
    Extension(pool): Extension<SqlitePool>,
    base: BaseTemplate,
) -> Result<Markup, AppError> {
    let top_statements = db::top_statements(&pool).await?;
    let content = html! {
        div class="mb-10 flex justify-center" {
            input
                type="search"
                name="typed_query"
                placeholder="Find Questions"
                class="dark:text-black w-full rounded-full px-7 py-4 border border-1 border-gray-400"
                minLength="1"
                hx-validate="true"
                hx-target="#results"
                hx-post="/search"
                hx-trigger="keyup changed delay:100ms, keydown[key=='Enter']"
                data-testid="create-statement-field"
                {}
        }
        div id="results" {
            h2 class="mb-4 text-xl" { "Controversial Questions" }
            @for statement in top_statements.iter() {
                div class="mb-5 rounded-lg shadow bg-white dark:bg-slate-700 flex" {
                    (inline_statement_content(statement, None, true, &maybe_user, &pool).await?)
                    (inline_statement_piechart(statement.id, &pool).await?)
                }
            }
        }

    };
    Ok(base.title("Propolis").content(content).render())
}

#[derive(Deserialize)]
pub struct SearchForm {
    typed_query: String,
}

pub async fn search_results(
    maybe_user: Option<User>,
    Extension(pool): Extension<SqlitePool>,
    Form(form): Form<SearchForm>,
) -> Result<Markup, AppError> {
    let statements = db::search_statement(form.typed_query.as_str(), &pool).await?;
    Ok(html! {
        @for search_result_statement in &statements {
            div class="mb-5 rounded-lg shadow bg-white dark:bg-slate-700 flex" {
                (inline_statement_content(&search_result_statement.statement_highlighted(), None, true, &maybe_user, &pool).await?)
                (inline_statement_piechart(search_result_statement.id, &pool).await?)
                (inline_statement_vote_fetch(search_result_statement.id, &maybe_user, &pool).await?)
            }
        }
    })
}
