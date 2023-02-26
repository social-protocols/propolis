use crate::structs::User;
use crate::{db::random_statement_id, error::Error};

use axum::{response::Redirect, Extension};
use sqlx::SqlitePool;

pub async fn redirect_to_next_statement(
    existing_user: Option<User>,
    Extension(pool): Extension<SqlitePool>,
) -> Result<Redirect, Error> {
    let statement_id: Option<i64> = match existing_user {
        Some(user) => user.next_statement_for_user(&pool).await?,
        None => random_statement_id(&pool).await?,
    };

    Ok(match statement_id {
        Some(id) => Redirect::to(format!("/statement/{}", id).as_str()),
        None => Redirect::to("/statement/0"), // TODO
    })
}

pub async fn index(
    existing_user: Option<User>,
    Extension(pool): Extension<SqlitePool>,
) -> Result<Redirect, Error> {
    Ok(redirect_to_next_statement(existing_user, Extension(pool)).await?)
}
