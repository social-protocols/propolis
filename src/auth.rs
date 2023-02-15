use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use serde::Serialize;
use sqlx::SqlitePool;
use tower_cookies::{Cookie, Cookies};

#[derive(Serialize, sqlx::FromRow)]
pub struct User {
    pub id: i64,
    pub secret: String,
}

pub async fn ensure_auth(cookies: &Cookies, pool: &SqlitePool) -> User {
    if let Some(secret) = cookies.get("secret") {
        let secret = secret.value().to_string();
        let query = sqlx::query_as!(
            User,
            "SELECT id, secret from users WHERE secret = ?",
            secret
        );

        let result = query.fetch_optional(pool).await.expect("Must be valid");

        match result {
            Some(result) => User {
                id: result.id,
                secret: secret.to_string(),
            },
            None => {
                let user = create_user(pool).await;
                cookies.add(Cookie::new("secret", user.secret.to_owned()));

                user
            }
        }
    } else {
        let user = create_user(pool).await;
        cookies.add(Cookie::new("secret", user.secret.to_owned()));

        user
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
