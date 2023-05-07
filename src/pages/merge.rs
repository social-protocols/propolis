use super::base::BaseTemplate;

use crate::structs::User;
use crate::{auth::change_auth_cookie, error::AppError};

use axum::{extract::Path, Extension, Form};
use maud::{html, Markup};
use serde::Deserialize;
use sqlx::SqlitePool;
use tower_cookies::Cookies;

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
    Path(secret): Path<String>,
    cookies: Cookies,
    Extension(pool): Extension<SqlitePool>,
    base: BaseTemplate,
) -> Result<Markup, AppError> {
    let user = User::get_or_create(&cookies, &pool).await?;
    let num_votes = user.num_votes(&pool).await?;
    let num_statements = user.num_statements(&pool).await?;

    let current_secret = user.secret.to_owned();
    let new_secret = secret;
    if new_secret == current_secret {
        // TODO: how to use anyhow::ensure! here?
        return Err(AppError(anyhow::anyhow!("Cannot merge with same secret")));
    }

    let content = html! {
        h1 { "Merge" }
        form hx-post={"/merge/"(new_secret)} {
            fieldset {
            big { "Switch from " code { (current_secret) } " to " code { (new_secret) } "?" }
            p { "This will..." }
            ul {
                li { "move " (num_votes) " votes and " (num_statements) " statements" }
                li { "switch to new secret " code { (new_secret) } }
                li { "delete old secret " code { (current_secret) } }
            }
            p { "Continue?" }
            button name="value" type="submit" value="Yes" { "yes" }
            button name="value" type="cancel" value="No" { "no" }
            button name="value" type="submit" value="YesWithoutMerge" { "yes, but skip merge" }
        }
        }
    };

    Ok(base.title("Merge accounts").content(content).into())
}

pub async fn merge_post(
    user: User,
    cookies: Cookies,
    Path(secret): Path<String>,
    Extension(pool): Extension<SqlitePool>,
    Form(merge): Form<MergeForm>,
) -> Result<Markup, AppError> {
    Ok(match User::from_secret(secret.as_str(), &pool).await? {
        Some(new_user) => {
            if user.id == new_user.id {
                return Ok(html! {"Merge aborted: same user"});
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
                    change_auth_cookie(new_user.secret.as_str(), &cookies);
                    tx.commit().await.expect("Transaction commit failed");

                    html! {"Merge successful"}
                }

                MergeAnswer::No => html! {"Merge aborted."},
            }
        }

        None => html! {"Target user does not exist."},
    })
}
