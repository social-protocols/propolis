use super::base::{get_base_template, BaseTemplate, GenericViewTemplate, WarningDialog };
use crate::{auth::User, error::Error, util::base_url};
use askama::Template;
use maud::html;

use axum::{
    response::{Html, Redirect},
    Extension, Form,
};
use http::HeaderMap;
use serde::Deserialize;
use sqlx::SqlitePool;
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

fn html(base: &BaseTemplate, merge_url: &str, qr_code: &str) -> String {
    html! {
        h1 { "Options" }
        fieldset {
            p { "Use this QR Code on another device to switch it to this account:" }
            img id="qr-code" src=(format!("data:image/svg+xml;base64,{}", qr_code ));
            br;
            small {
                "Or open ";
                a href=( merge_url ) { ( merge_url ) }
                "on your other device"
            }
        }
        form id="theme-form" method="post" action="/options" _="on test call me.requestSubmit()" {
            fieldset {
                label for="theme" { "theme" }
                select id="theme" name="theme" _="on change send test to #theme-form" {
                    option value="light" selected=[{ if base.theme == "light" { Some(1) } else { None } }] { "Light" }
                    option value="dark" selected=[{ if base.theme == "dark" { Some(1) } else { None } }] { "Dark" }
                }
            }
        }
    }
    .into_string()
}

pub async fn options(
    headers: HeaderMap,
    maybe_user: Option<User>,
    cookies: Cookies,
    Extension(pool): Extension<SqlitePool>,
) -> Result<Html<String>, Error> {
    let title = Some("Options");

    Ok(match maybe_user {
        Some(user) => {
            let merge_url = format!("{}/merge/{}", base_url(&headers), &user.secret);
            let base = get_base_template(cookies, Extension(pool));
            let content = html(&base, &merge_url, qr_code_base64(&merge_url).as_str());
            let template = GenericViewTemplate { base, content: content.as_str(), title };

            Html(template.render()?)
        }
        None => Html(
            GenericViewTemplate {
                base: get_base_template(cookies, Extension(pool)),
                title,
                content: String::from(WarningDialog {
                    msg: "Options disabled until you cast your first vote.",
                    ..Default::default()
                }).as_str(),
            }
            .render()?,
        ),
    })
}

pub async fn options_post(
    cookies: Cookies,
    Form(options_form): Form<OptionsForm>,
) -> Result<Redirect, Error> {
    cookies.add(Cookie::new("theme", options_form.theme));

    Ok(Redirect::to("/options"))
}
