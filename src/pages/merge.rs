use super::base::{get_base_template, BaseTemplate};
use crate::auth::{ensure_auth, User};

use askama::Template;
use axum::{
    response::{Html, Redirect},
    Extension, Form, extract::Path,
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
}

pub async fn merge(
    user: User,
    Path(secret): Path<String>,
    cookies: Cookies,
    Extension(pool): Extension<SqlitePool>) -> Html<String> {

    let template = MergeTemplate {
        base: get_base_template(cookies, Extension(pool)),
        current_secret: user.secret.to_owned(),
        new_secret: secret,
    };

    Html(template.render().unwrap())
}

#[derive(Deserialize, Debug)]
enum MergeAnswer {
    Yes,
    No,
    YesWithoutMerge
}

#[derive(Deserialize)]
pub struct MergeForm {
    value: MergeAnswer,
}

pub async fn merge_post(
    user: User,
    cookies: Cookies,
    Extension(pool): Extension<SqlitePool>,
    Form(merge): Form<MergeForm>,
) -> Html<String> {

    println!("{:?}", merge.value);
    Html("You are now merged with ?".to_string())
}
