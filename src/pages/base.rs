use maud::{html, Markup, PreEscaped, DOCTYPE};
use tower_cookies::{Cookie, Cookies};

fn render_base(theme: String, title: Option<String>, content: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" data-theme=(theme) {
            head {
                meta name="viewport" content="width=device-width, initial-scale=1.0";

                link rel="apple-touch-icon" sizes="180x180" href="/apple-touch-icon.png";
                link rel="icon" type="image/png" sizes="32x32" href="/favicon-32x32.png";
                link rel="icon" type="image/png" sizes="16x16" href="/favicon-16x16.png";
                link rel="manifest" href="/site.webmanifest";
                link rel="mask-icon" href="/safari-pinned-tab.svg" color="#da8d2b";
                link rel="stylesheet" type="text/css" href="/css/apexcharts.css";

                meta name="msapplication-TileColor" content="#2b5797";
                meta name="theme-color" content="#ffffff";

                style { (PreEscaped(include_str!("../../templates/css/classless.css"))) }
                style { (PreEscaped(include_str!("../../templates/css/theme.css"))) }
                style { (PreEscaped(include_str!("../../templates/css/main.css"))) }
                script src="/js/utils.js" {}
                script src="/js/htmx.min.js" {}
                script src="/js/_hyperscript.min.js" {}
                script src="/js/apexcharts.min.js" {}

                title { (title.unwrap_or("Propolis".to_string())) }
            }
            body {
                nav {
                    ul {
                        li { a href="/" { "Home" } }
                        li { a href="/new" { "Add a statement" } }
                        li { a href="/submissions" { "Your submissions" } }
                        li class="float-right" { a href="/options" { "⚙" } }
                    }
                }
                div id="content" {
                    (content)
                }
                div _=r#"on delayedRedirectTo(value) from body
            wait 2s
            call window.location.replace(value)"# {}
            }
        }
    }
}

pub fn base(cookies: Cookies, title: Option<String>, content: Markup) -> Markup {
    let theme = cookies
        .get("theme")
        .unwrap_or(Cookie::new("theme", "light"));

    render_base(theme.value().to_string(), title, content)
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
