use crate::db::random_statement_id;
use crate::error::AppError;
use crate::structs::User;

use anyhow::Result;
use axum::{response::Redirect, Extension};
use sqlx::SqlitePool;

pub async fn next_statement_id(
    existing_user: Option<User>,
    Extension(pool): Extension<SqlitePool>,
) -> Result<Option<i64>> {
    Ok(match existing_user {
        Some(user) => user.next_statement_for_user(&pool).await?,
        None => random_statement_id(&pool).await?,
    })
}

pub async fn redirect_to_next_statement(
    existing_user: Option<User>,
    Extension(pool): Extension<SqlitePool>,
) -> Result<Redirect, AppError> {
    let statement_id = next_statement_id(existing_user, Extension(pool)).await?;

    Ok(match statement_id {
        Some(id) => Redirect::to(format!("/statement/{id}").as_str()),
        None => Redirect::to("/statement/0"), // TODO
    })
}

pub async fn index(
    existing_user: Option<User>,
    Extension(pool): Extension<SqlitePool>,
) -> Result<Redirect, AppError> {
    Ok(redirect_to_next_statement(existing_user, Extension(pool)).await?)
}
