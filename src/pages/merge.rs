use super::base::{get_base_template, BaseTemplate};
use crate::auth::User;
use crate::db::UserQueries;

use askama::Template;
use axum::{
    extract::Path,
    response::Html,
    Extension, Form,
};
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

#[derive(Deserialize, Debug)]
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
) -> Html<String> {
    let num_votes = user.num_votes(&pool).await;
    let num_statements = user.num_statements(&pool).await;

    let template = MergeTemplate {
        base: get_base_template(cookies, Extension(pool)),
        num_votes,
        num_statements,
        current_secret: user.secret.to_owned(),
        new_secret: secret,
    };

    Html(template.render().unwrap())
}

pub async fn merge_post(
    user: User,
    Path(new_secret): Path<String>,
    Extension(pool): Extension<SqlitePool>,
    Form(merge): Form<MergeForm>,
) -> Html<String> {

    match crate::auth::user_for_secret(new_secret, &pool).await {

        Some(new_user) => {

            match merge.value {

                MergeAnswer::Yes | MergeAnswer::YesWithoutMerge => {

                    user.move_content_to(&new_user, &pool).await;
                    user.delete(&pool).await;

                    Html("You are now merged with ?".to_string())
                }

                MergeAnswer::No => {
                    Html("Merge aborted.".to_string())
                }

            }
        }

        None => {
            Html("Target user does not exist.".to_string())
        }
    }


}
