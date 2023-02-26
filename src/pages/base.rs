use askama::Template;
use axum::Extension;
use sqlx::SqlitePool;
use tower_cookies::{Cookie, Cookies};

pub struct BaseTemplate {
    pub theme: String,
}

#[derive(Template)]
#[template(path = "generic_view.j2")]
pub struct GenericViewTemplate {
    pub base: BaseTemplate,
    pub content: String,
}

pub fn get_base_template(cookies: Cookies, Extension(_): Extension<SqlitePool>) -> BaseTemplate {
    let theme = cookies
        .get("theme")
        .unwrap_or(Cookie::new("theme", "light"));

    BaseTemplate {
        theme: theme.value().to_string(),
    }
}
