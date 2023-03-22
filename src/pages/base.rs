use maud::{html, Markup, DOCTYPE};
use tower_cookies::{Cookie, Cookies};

use crate::{structs::User, StaticAsset};

fn render_base(
    theme: String,
    title: Option<String>,
    user: &Option<User>,
    content: Markup,
) -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" data-theme=(theme) {
            head {
                // TODO: link preview
                meta name="viewport" content="width=device-width, initial-scale=1.0";

                link rel="apple-touch-icon" sizes="180x180" href="/apple-touch-icon.png";
                link rel="icon" type="image/png" sizes="32x32" href="/favicon-32x32.png";
                link rel="icon" type="image/png" sizes="16x16" href="/favicon-16x16.png";
                link rel="manifest" href="/site.webmanifest";
                link rel="mask-icon" href="/safari-pinned-tab.svg" color="#da8d2b";

                meta name="msapplication-TileColor" content="#2b5797";
                meta name="theme-color" content="#ffffff";

                @for file in StaticAsset::iter().filter(|path| path.starts_with("css/")) {
                    link rel="stylesheet" href={"/"(file)} {}
                }

                @for file in StaticAsset::iter().filter(|path| path.starts_with("js/")) {
                    script src={"/"(file)} {}
                }

                title { (title.unwrap_or("Propolis".to_string())) }
            }
            body {
                nav {
                    ul style="display:flex" {
                        li { a href="/" { "Home" } }
                        li { a href="/new" { "Add a statement" } }
                        li  style="margin-right: auto" { a href="/subscriptions" { "My Subscriptions" } }
                        // first 4 characters of user id
                        @if let Some(user) = user {
                            li {
                                span style="margin-right: 0.5em" { "👤" }
                                (user.secret.chars().take(4).collect::<String>())
                            }
                        }
                        li { a href="/options" { "⚙" } }
                    }
                }
                div id="content" {
                    (content)
                }
            }
        }
    }
}

pub fn base(
    cookies: Cookies,
    title: Option<String>,
    user: &Option<User>,
    content: Markup,
) -> Markup {
    let theme = cookies
        .get("theme")
        .unwrap_or(Cookie::new("theme", "light"));

    render_base(theme.value().to_string(), title, user, content)
}

/// Presents a warning dialog to the user
pub fn warning_dialog(msg: &str, caption: Option<&str>) -> Markup {
    html!(
        div.warn.card {
            p { (caption.unwrap_or("Warning")) }
            p { (msg) }
        }
    )
}
