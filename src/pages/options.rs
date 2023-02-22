use crate::auth::ensure_auth;

use askama::Template;
use axum::{response::Html, Extension};
use sqlx::SqlitePool;
use tower_cookies::Cookies;

use qrcode::QrCode;
use qrcode::render::svg;

use base64::{Engine as _, engine::general_purpose};

#[derive(Template)]
#[template(path = "options.j2")]
struct OptionsTemplate<'a> {
    user_id: i64,
    secret: &'a String,
    qr_code: String,
}

pub fn qr_code_base64(code: &String) -> String {
    let code = QrCode::new(code.as_bytes()).unwrap();

    general_purpose::STANDARD_NO_PAD.encode(code.render::<svg::Color>().build())
}

pub async fn options(cookies: Cookies, Extension(pool): Extension<SqlitePool>) -> Html<String> {
    let user = ensure_auth(&cookies, &pool).await;

    let template = OptionsTemplate {
        user_id: user.id,
        secret: &user.secret,
        qr_code: qr_code_base64(&user.secret)
    };

    Html(template.render().unwrap())
}
