//! Authentication & user management

use anyhow::Result;
use async_trait::async_trait;
use axum::extract::FromRequestParts;
use axum::http::StatusCode;
use axum::Extension;
use http::request::Parts;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use sqlx::SqlitePool;
use tower_cookies::cookie::time::Duration;
use tower_cookies::{Cookie, Cookies};

use crate::structs::User;

const COOKIE_MAX_AGE: Duration = Duration::days(10 * 365);

impl User {
    /// If logged in with a secret, will return a [User]
    pub async fn from_cookies(cookies: &Cookies, pool: &SqlitePool) -> Result<Option<Self>> {
        Ok(match cookies.get("secret") {
            Some(cookie) => User::from_secret(cookie.value(), pool).await?,
            None => None,
        })
    }

    /// returns [User] via secret
    pub async fn from_secret(secret: &str, pool: &SqlitePool) -> Result<Option<Self>> {
        Ok(sqlx::query_as!(
            User,
            "SELECT id, secret from users WHERE secret = ?",
            secret
        )
        .fetch_optional(pool)
        .await?)
    }

    /// returns logged in [User] or creates a new one and returns that
    pub async fn get_or_create(cookies: &Cookies, pool: &SqlitePool) -> Result<User> {
        let existing_user: Option<User> = User::from_cookies(cookies, pool).await?;

        Ok(match existing_user {
            Some(user) => user,
            None => {
                let user = User::create(pool).await?;
                let cookie = Cookie::build("secret", user.secret.to_owned())
                    .max_age(COOKIE_MAX_AGE)
                    .finish();
                cookies.add(cookie);

                user
            }
        })
    }

    /// Creates a new [User] inside the database and return it
    pub async fn create(pool: &SqlitePool) -> Result<User> {
        let secret = generate_secret();
        let user =
            sqlx::query_as::<_, User>("INSERT INTO users (secret) VALUES (?) RETURNING id, secret")
                .bind(secret)
                .fetch_one(pool)
                .await?;

        Ok(user)
    }
}

/// Changes the cookie containing the secret to a different value
pub fn change_auth_cookie(secret: &str, cookies: &Cookies) {
    if let Some(mut cookie) = cookies.get("secret") {
        // copy old cookie, but also set path, since it may come from e.g. /merge
        cookie.set_value(secret);
        cookie.set_path("/");
        cookie.set_max_age(COOKIE_MAX_AGE);
        cookies.add(cookie.into_owned());
    }
}

fn generate_secret() -> String {
    // TODO: check if used already
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(16)
        .map(char::from)
        .collect()
}

#[async_trait]
impl<S> FromRequestParts<S> for User
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        use axum::RequestPartsExt;
        let Extension(pool) = parts
            .extract::<Extension<SqlitePool>>()
            .await
            .expect("Unable to get sqlite connection");
        let cookies = parts
            .extract::<Cookies>()
            .await
            .expect("Unable to get cookies");

        match User::from_cookies(&cookies, &pool).await {
            Ok(result) => result.ok_or((StatusCode::UNAUTHORIZED, "Unauthorized")),
            Err(_) => Err((StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")),
        }
    }
}
