use super::base::{get_base_template, BaseTemplate};
use crate::auth::ensure_auth;

use askama::Template;
use axum::{
    response::{Html, Redirect},
    Extension, Form,
};
use serde::Deserialize;
use sqlx::SqlitePool;
use tower_cookies::{Cookie, Cookies};

use qrcode::render::svg;
use qrcode::QrCode;

use base64::{engine::general_purpose, Engine as _};

#[derive(Template)]
#[template(path = "options.j2")]
struct OptionsTemplate<'a> {
    base: BaseTemplate,
    secret: &'a String,
    qr_code: String,
}

#[derive(Deserialize)]
pub struct OptionsForm {
    theme: String,
}

pub fn qr_code_base64(code: &String) -> String {
    let code = QrCode::new(code.as_bytes()).unwrap();

    general_purpose::STANDARD_NO_PAD.encode(code.render::<svg::Color>().build())
}

pub async fn options(cookies: Cookies, Extension(pool): Extension<SqlitePool>) -> Html<String> {
    let user = ensure_auth(&cookies, &pool).await;

    let template = OptionsTemplate {
        base: get_base_template(cookies, Extension(pool)),
        secret: &user.secret,
        qr_code: qr_code_base64(&user.secret),
    };

    Html(template.render().unwrap())
}

pub async fn options_post(cookies: Cookies, Form(options_form): Form<OptionsForm>) -> Redirect {
    cookies.add(Cookie::new("theme", options_form.theme));

    Redirect::to("/options")
}
