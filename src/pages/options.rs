use super::base::{warning_dialog, BaseTemplate};
use crate::error::AppError;
use crate::structs::User;
use crate::util::base_url;
use maud::{html, Markup};

use anyhow::Result;

use http::HeaderMap;

use qrcode::render::svg;
use qrcode::QrCode;

use base64::{engine::general_purpose, Engine as _};

pub fn qr_code_base64(code: &String) -> String {
    let code = QrCode::new(code.as_bytes()).unwrap();

    general_purpose::STANDARD_NO_PAD.encode(code.render::<svg::Color>().build())
}

fn html(merge_url: &str, qr_code: &str) -> Markup {
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
        // TODO: save theme in localstorage
        // fieldset {
        //     label for="theme" { "theme" }
        //     select id="theme" name="theme" x-on:input="$refs.themeForm.submit()" {
        //         option value="light" selected[theme == "light"] { "Light" }
        //         option value="dark" selected[theme == "dark"] { "Dark" }
        //     }
        // }
    }
}

pub async fn options(
    headers: HeaderMap,
    maybe_user: Option<User>,
    base: BaseTemplate,
) -> Result<Markup, AppError> {
    let title = "Options";

    match maybe_user {
        Some(user) => {
            let merge_url = format!("{}/merge/{}", base_url(&headers), &user.secret);
            let content = html(&merge_url, qr_code_base64(&merge_url).as_str());
            Ok(base.title(title).content(content).into())
        }
        None => Ok(base
            .title(title)
            .content(warning_dialog(
                "Options disabled until you cast your first vote.",
                None,
            ))
            .into()),
    }
}
