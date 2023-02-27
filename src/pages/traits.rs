use std::collections::HashMap;

use super::base::{get_base_template, BaseTemplate, GenericViewTemplate, WarningDialog};
use crate::structs::User;
use crate::{error::Error, util::base_url};
use maud::html;

use axum::{
    response::{Html, Redirect},
    Extension, Form,
};
use sqlx::SqlitePool;
use tower_cookies::{Cookie, Cookies};


fn html() -> String {
    html! {
        table width="100%" {
            thead {
                tr {
                    th { "Left" }
                    th ;
                    th { "Right" }
                }
            }
            tbody {
                @for (category, trait_extremes) in HashMap::from(
                    [("Political", vec![("Conservative", "Liberal"),
                                        ("Capitalistic", "Socialistic"),]),
                     ("Personal", vec![("Extraverted", "Intraverted")])]) {
                    tr {
                        th { (category) }
                        th;
                        th;
                    }
                        @for (left_extreme, right_extreme) in trait_extremes {
                            tr {
                                td {(left_extreme)}
                                td;
                                td {(right_extreme)}}
                        }
                }
            }
        }
    }
    .into_string()
}

pub async fn traits(
    user: User,
    cookies: Cookies,
    Extension(pool): Extension<SqlitePool>,
) -> Result<Html<String>, Error> {
    let title = Some("Traits");

    let base = get_base_template(cookies, Extension(pool));
    let content = html();
    GenericViewTemplate {
        base,
        content: content.as_str(),
        title,
    }
    .into()
}
