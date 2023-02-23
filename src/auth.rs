use axum::Extension;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use serde::Serialize;
use sqlx::SqlitePool;
use tower_cookies::{Cookie, Cookies};
use axum::extract::FromRequestParts;
use axum::http::StatusCode;
use http::request::Parts;
use async_trait::async_trait;


#[derive(Serialize, sqlx::FromRow, Debug)]
pub struct User {
    pub id: i64,
    pub secret: String,
}

pub async fn logged_in_user(cookies: &Cookies, pool: &SqlitePool) -> Option<User> {
    match cookies.get("secret") {
        Some(cookie) => user_for_secret(cookie.value().to_string(), pool).await,
        None => None,
    }
}

pub async fn user_for_secret(secret: String, pool: &SqlitePool) -> Option<User> {
    sqlx::query_as!(
        User,
        "SELECT id, secret from users WHERE secret = ?",
        secret
    )
    .fetch_optional(pool)
    .await
    .expect("Must be valid")
    .map(|user| User {
        id: user.id,
        secret: user.secret.to_string(),
    })
}

pub async fn ensure_auth(cookies: &Cookies, pool: &SqlitePool) -> User {
    let existing_user: Option<User> = logged_in_user(cookies, pool).await;

    match existing_user {
        Some(user) => user,
        None => {
            let user = create_user(pool).await;
            cookies.add(Cookie::new("secret", user.secret.to_owned()));

            user
        }
    }
}

async fn create_user(pool: &SqlitePool) -> User {
    let secret = generate_secret();
    let user =
        sqlx::query_as::<_, User>("INSERT INTO users (secret) VALUES (?) RETURNING id, secret")
            .bind(secret)
            .fetch_one(pool)
            .await
            .expect("Must be valid");

    user
}

fn generate_secret() -> String {
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

    async fn from_request_parts(
        parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {

        use axum::RequestPartsExt;
        let Extension(pool) = parts.extract::<Extension<SqlitePool>>()
            .await
            .expect("Unable to get sqlite connection");
        let cookies = parts.extract::<Cookies>()
            .await
            .expect("Unable to get sqlite connection");

        logged_in_user(&cookies, &pool).await
            .ok_or((StatusCode::UNAUTHORIZED, "Unauthorized"))
    }
}
