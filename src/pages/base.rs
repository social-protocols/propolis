use axum::Extension;
use sqlx::SqlitePool;
use tower_cookies::{Cookie, Cookies};

pub struct BaseTemplate {
    pub theme: String,
}

pub fn get_base_template(cookies: Cookies, Extension(_): Extension<SqlitePool>) -> BaseTemplate {
    let theme = cookies
        .get("theme")
        .unwrap_or(Cookie::new("theme", "light"));

    BaseTemplate {
        theme: theme.value().to_string(),
    }
}
