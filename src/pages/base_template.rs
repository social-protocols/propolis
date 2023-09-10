use async_trait::async_trait;
use axum::extract::FromRequestParts;
use http::HeaderMap;
use maud::{html, Markup, DOCTYPE};
use tower_cookies::Cookies;

use crate::{
    http_static::StaticAsset,
    structs::{PageMeta, User},
    util::base_url,
};

fn render_base_template(
    title: Option<String>,
    user: &Option<User>,
    content: Markup,
    headers: &HeaderMap,
    page_meta: Option<PageMeta>,
) -> Markup {
    html! {
        (DOCTYPE)
        // hx-boost makes the navigation faster by making links and forms use AJAX:
        // https://htmx.org/attributes/hx-boost/
        html lang="en" hx-boost="true" {
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

                meta property="og:type" content="website";

                @if let Some(page_meta) = page_meta {
                    @if let Some(title) = page_meta.title {
                        meta property="og:title" content=(title);
                    }
                    @if let Some(description) = page_meta.description {
                        meta property="og:description" content=(description);
                    }
                    @if let Some(url) = page_meta.url {
                        meta property="og:url" content=(url);
                    } @else {
                        meta property="og:url" content=(base_url(headers));
                    }
                }

                @for file in StaticAsset::iter().filter(|path| path.starts_with("css/")) {
                    link rel="stylesheet" href={"/"(file)} {}
                }

                @for file in StaticAsset::iter().filter(|path| path.starts_with("js/")) {
                    script src={"/"(file)} {}
                }
                @for file in StaticAsset::iter().filter(|path| path.starts_with("js-defer/")) {
                    script defer src={"/"(file)} {}
                }

                title { (title.unwrap_or("Propolis".to_string())) }
            }
            body class="bg-slate-100 dark:bg-slate-800 dark:text-white" {
                nav class="px-5 py-3" {
                    ul class="flex gap-6" {
                        li { a href="/" data-testid="nav-home" { "Home" } }
                        li { a href="/statement" data-testid="nav-home" { "Vote" } }
                        li { a href="/new" data-testid="nav-add-statement" { "Ask Question" } }
                        li  class="mr-auto" { a href="/subscriptions" data-testid="nav-my-subscriptions" { "My Subscriptions" } }
                        // first 4 characters of user id
                        @if let Some(user) = user {
                            li {
                                a href="/user" class="font-mono" {
                                    @let user_icon = "👤";
                                    span class="mr-1" { (user_icon) }
                                    (user.secret.chars().take(4).collect::<String>())
                                }
                            }
                        }
                        li {
                            a href="/options" {
                                @let cog_icon = "⚙";
                                (cog_icon)
                            }
                        }
                    }
                }
                div class="p-5" {
                    (content)
                }
            }
        }
    }
}

#[derive(Clone)]
pub struct BaseTemplate {
    pub user: Option<User>,
    pub cookies: Cookies,
    pub headers: HeaderMap,
    pub title: Option<String>,
    pub content: Markup,
    pub page_meta: Option<PageMeta>,
}

impl BaseTemplate {
    /// Set the page title
    pub fn title(mut self, s: &str) -> Self {
        self.title = Some(s.into());
        self
    }
    /// Set the page content
    pub fn content(mut self, c: Markup) -> Self {
        self.content = c;
        self
    }
    /// Set the page meta data
    pub fn page_meta(mut self, m: PageMeta) -> Self {
        self.page_meta = Some(m);
        self
    }
    /// Set the page meta data
    pub fn page_meta_opt(mut self, m: Option<PageMeta>) -> Self {
        self.page_meta = m;
        self
    }
    /// Render BaseTemplate into markup
    pub fn render(self) -> Markup {
        render_base_template(
            self.title,
            &self.user,
            self.content,
            &self.headers,
            self.page_meta,
        )
    }
}

impl From<BaseTemplate> for Markup {
    fn from(value: BaseTemplate) -> Self {
        value.render()
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for BaseTemplate
where
    S: Send + Sync,
{
    type Rejection = (http::StatusCode, &'static str);

    async fn from_request_parts(
        parts: &mut http::request::Parts,
        _: &S,
    ) -> Result<Self, Self::Rejection> {
        use axum::RequestPartsExt;
        let cookies = parts
            .extract::<Cookies>()
            .await
            .expect("Unable to get cookies");
        let user = parts
            .extract::<Option<User>>()
            .await
            .expect("Unable to get user");
        let headers = parts
            .extract::<HeaderMap>()
            .await
            .expect("Unable to get headers");

        Ok(BaseTemplate {
            user,
            cookies,
            headers,
            title: None,
            content: html! {},
            page_meta: None,
        })
    }
}
