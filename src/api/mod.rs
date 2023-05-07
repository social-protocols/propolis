use anyhow::anyhow;
use anyhow::Result;
use axum::debug_handler;
use axum::extract::Path;
use axum::headers::authorization::Bearer;
use axum::headers::Authorization;

use axum::Extension;
use axum::Json;
use axum::TypedHeader;

use serde::Deserialize;
use serde::Serialize;
use sqlx::SqlitePool;

use crate::db::get_statement;

use crate::structs::Vote;
use crate::{error::AppError, structs::User};

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiStatement {
    pub id: i64,
    pub text: String,
}

pub async fn create_user(Extension(pool): Extension<SqlitePool>) -> Result<String, AppError> {
    let user = User::create(&pool).await?;
    Ok(user.secret)
}

// TODO: extract user with middleware
pub async fn next_statement(
    Extension(pool): Extension<SqlitePool>,
    TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
) -> Result<Json<ApiStatement>, AppError> {
    let secret = bearer.token();
    let user = User::from_secret(secret, &pool)
        .await?
        .ok_or(anyhow!("Unauthorized"))?;

    let statement_id = user
        .next_statement_for_user(&pool)
        .await?
        .ok_or(anyhow!("No more statements"))?;
    let statement = get_statement(statement_id, &pool).await?;
    Ok(Json(ApiStatement {
        id: statement.id,
        text: statement.text,
    }))
}

#[debug_handler]
pub async fn statement_vote(
    Path(statement_id): Path<i64>,
    Extension(pool): Extension<SqlitePool>,
    TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
    Json(vote): Json<Vote>,
) -> Result<(), AppError> {
    let secret = bearer.token();
    let user = User::from_secret(secret, &pool)
        .await?
        .ok_or(anyhow!("Unauthorized"))?;

    user.vote(statement_id, vote, &pool).await?;
    Ok(())
}
