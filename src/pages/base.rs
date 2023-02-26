use askama::Template;
use axum::Extension;
use maud::html;
use sqlx::SqlitePool;
use tower_cookies::{Cookie, Cookies};

pub struct BaseTemplate {
    pub theme: String,
}

#[derive(Template)]
#[template(path = "generic_view.j2")]
pub struct GenericViewTemplate<'a> {
    pub base: BaseTemplate,
    pub content: &'a str,
    pub title: Option<&'a str>,
}

pub fn get_base_template(cookies: Cookies, Extension(_): Extension<SqlitePool>) -> BaseTemplate {
    let theme = cookies
        .get("theme")
        .unwrap_or(Cookie::new("theme", "light"));

    BaseTemplate {
        theme: theme.value().to_string(),
    }
}

/// Presents a warning dialog to the user
pub struct WarningDialog<'a> {
    pub msg: &'a str,
    pub caption: Option<&'a str>,
}

impl<'a> Default for WarningDialog<'a> {
    fn default() -> Self {
        WarningDialog {
            msg: "Nothing to see here",
            caption: Some("Warning")
        }
    }
}

impl<'a> From<WarningDialog<'a>> for String {
    fn from(dlg: WarningDialog<'a>) -> String {
        html!(
            div.warn.card {
                p { (dlg.caption.unwrap_or("Warning")) }
                p { (dlg.msg) }
            }
        ).into_string()
    }
}
