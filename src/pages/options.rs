use super::base::{base, warning_dialog};
use crate::error::AppError;
use crate::structs::User;
use crate::util::base_url;
use maud::{html, Markup};

use anyhow::Result;
use axum::{response::Redirect, Form};
use http::HeaderMap;
use serde::Deserialize;
use tower_cookies::{Cookie, Cookies};

use qrcode::render::svg;
use qrcode::QrCode;

use base64::{engine::general_purpose, Engine as _};

#[derive(Deserialize)]
pub struct OptionsForm {
    theme: String,
}

pub fn qr_code_base64(code: &String) -> String {
    let code = QrCode::new(code.as_bytes()).unwrap();

    general_purpose::STANDARD_NO_PAD.encode(code.render::<svg::Color>().build())
}

fn html(theme: String, merge_url: &str, qr_code: &str) -> Markup {
    html! {
        fieldset {
            p { "Use this QR Code on another device to switch it to this account:" }
            img id="qr-code" src=(format!("data:image/svg+xml;base64,{qr_code}"));
            br;
            small {
                "Or open ";
                a href=( merge_url ) { ( merge_url ) }
                " on your other device"
            }
        }
        form x-data="" x-ref="themeForm" method="post" action="/options" {
            fieldset {
                label for="theme" { "theme" }
                select id="theme" name="theme" x-on:input="$refs.themeForm.submit()" {
                    option value="light" selected[theme == "light"] { "Light" }
                    option value="dark" selected[theme == "dark"] { "Dark" }
                }
            }
        }
    }
}

pub async fn options(
    headers: HeaderMap,
    maybe_user: Option<User>,
    cookies: Cookies,
) -> Result<Markup, AppError> {
    let title = Some("Options".to_string());

    match maybe_user {
        Some(user) => {
            let merge_url = format!("{}/merge/{}", base_url(&headers), &user.secret);
            let theme = cookies
                .get("theme")
                .map(|c| c.value().to_string())
                .unwrap_or_else(|| String::from("light"));
            let content = html(theme, &merge_url, qr_code_base64(&merge_url).as_str());
            Ok(base(cookies, title, &Some(user), content, &headers, None))
        }
        None => Ok(base(
            cookies,
            title,
            &maybe_user,
            warning_dialog("Options disabled until you cast your first vote.", None),
            &headers,
            None,
        )),
    }
}

pub async fn options_post(
    cookies: Cookies,
    Form(options_form): Form<OptionsForm>,
) -> Result<Redirect, AppError> {
    cookies.add(Cookie::new("theme", options_form.theme));

    Ok(Redirect::to("/options"))
}
