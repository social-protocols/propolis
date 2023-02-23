use super::base::{get_base_template, BaseTemplate};
use crate::auth::User;

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

#[derive(Template)]
#[template(path = "empty_options.j2")]
struct EmptyOptionsTemplate {
    base: BaseTemplate,
}

#[derive(Deserialize)]
pub struct OptionsForm {
    theme: String,
}

pub fn qr_code_base64(code: &String) -> String {
    let code = QrCode::new(code.as_bytes()).unwrap();

    general_purpose::STANDARD_NO_PAD.encode(code.render::<svg::Color>().build())
}

pub async fn options(
    maybe_user: Option<User>,
    cookies: Cookies,
    Extension(pool): Extension<SqlitePool>,
) -> Html<String> {
    match maybe_user {
        Some(user) => {
            let template = OptionsTemplate {
                base: get_base_template(cookies, Extension(pool)),
                secret: &user.secret,
                qr_code: qr_code_base64(&user.secret),
            };

            Html(template.render().unwrap())
        }
        None => Html(
            EmptyOptionsTemplate {
                base: get_base_template(cookies, Extension(pool)),
            }
            .render()
            .unwrap(),
        ),
    }
}

pub async fn options_post(cookies: Cookies, Form(options_form): Form<OptionsForm>) -> Redirect {
    cookies.add(Cookie::new("theme", options_form.theme));

    Redirect::to("/options")
}
