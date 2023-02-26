use super::base::{get_base_template, BaseTemplate};
use crate::auth::{switch_auth_cookie, User};
use crate::db::UserQueries;
use crate::error::Error;

use askama::Template;
use axum::{extract::Path, response::Html, Extension, Form};
use serde::Deserialize;
use sqlx::SqlitePool;
use tower_cookies::Cookies;

#[derive(Template)]
#[template(path = "merge.j2")]
struct MergeTemplate {
    base: BaseTemplate,
    current_secret: String,
    new_secret: String,
    num_votes: i32,
    num_statements: i32,
}

#[derive(Deserialize, Debug, PartialEq)]
enum MergeAnswer {
    Yes,
    No,
    YesWithoutMerge,
}

#[derive(Deserialize)]
pub struct MergeForm {
    value: MergeAnswer,
}

pub async fn merge(
    user: User,
    Path(secret): Path<String>,
    cookies: Cookies,
    Extension(pool): Extension<SqlitePool>,
) -> Result<Html<String>, Error> {
    let num_votes = user.num_votes(&pool).await?;
    let num_statements = user.num_statements(&pool).await?;

    let template = MergeTemplate {
        base: get_base_template(cookies, Extension(pool)),
        num_votes,
        num_statements,
        current_secret: user.secret.to_owned(),
        new_secret: secret,
    };

    Ok(Html(template.render().unwrap()))
}

pub async fn merge_post(
    user: User,
    cookies: Cookies,
    Path(secret): Path<String>,
    Extension(pool): Extension<SqlitePool>,
    Form(merge): Form<MergeForm>,
) -> Result<Html<String>, Error> {
    Ok(match User::from_secret(secret, &pool).await? {
        Some(new_user) => {
            if user.id == new_user.id {
                return Ok(Html("Merge aborted: same user".to_string()));
            }

            match merge.value {
                MergeAnswer::Yes | MergeAnswer::YesWithoutMerge => {
                    let tx = pool.begin().await.expect("Transaction begin failed");

                    if merge.value == MergeAnswer::Yes {
                        user.move_content_to(&new_user, &pool).await?;
                    } else {
                        user.delete_content(&pool).await?;
                    }

                    user.delete(&pool).await?;
                    switch_auth_cookie(new_user.secret, &cookies);
                    tx.commit().await.expect("Transaction commit failed");

                    Html("Merge successful".to_string())
                }

                MergeAnswer::No => Html("Merge aborted.".to_string()),
            }
        }

        None => Html("Target user does not exist.".to_string()),
    })
}
