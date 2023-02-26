use super::base::{get_base_template, BaseTemplate};
use crate::{error::Error, auth::User};

use askama::Template;
use axum::{
    response::{Html, Redirect},
    Extension, Form,
};
use serde::Deserialize;
use sqlx::SqlitePool;
use tower_cookies::Cookies;

#[derive(Template)]
#[template(path = "new_statement.j2")]
struct NewStatementTemplate {
    base: BaseTemplate,
}

pub async fn new_statement(
    cookies: Cookies,
    Extension(pool): Extension<SqlitePool>,
) -> Html<String> {
    let template = NewStatementTemplate {
        base: get_base_template(cookies, Extension(pool)),
    };

    Html(template.render().unwrap())
}

#[derive(Deserialize)]
pub struct AddStatementForm {
    statement_text: String,
}

pub async fn new_statement_post(
    cookies: Cookies,
    Extension(pool): Extension<SqlitePool>,
    Form(add_statement): Form<AddStatementForm>,
) -> Result<Redirect, Error> {
    let user = User::get_or_create(&cookies, &pool).await?;
    user.add_statement(add_statement.statement_text, &pool)
        .await?;

    Ok(Redirect::to("/"))
}
