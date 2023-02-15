use axum::{
    response::Html,
    routing::{get, post},
    Extension, Form, Router,
};
use dotenvy::dotenv;
use std::env;
use tower_cookies::{Cookie, CookieManagerLayer, Cookies};

use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use std::net::SocketAddr;

use askama::Template;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};

fn generate_secret() -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(16)
        .map(char::from)
        .collect()
}

// TODO: Login with user secret and cookie

#[tokio::main]
async fn main() {
    dotenv().expect(".env file not found");
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .unwrap();

    let app = Router::new()
        .route("/", get(index))
        .route("/", post(index_post))
        .layer(Extension(pool))
        .layer(CookieManagerLayer::new());

    let addr = SocketAddr::from(([127, 0, 0, 1], 8000));
    println!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[derive(Template)]
#[template(path = "index.j2")]
struct IndexTemplate<'a> {
    statement: &'a Option<Statement>,
}

#[derive(Serialize, sqlx::FromRow)]
struct Statement {
    id: i64,
    text: String,
}

#[derive(Deserialize)]
struct UserStatementVote {
    statement_id: i64,
    vote: i32,
}

async fn index_post(
    cookies: Cookies,
    Extension(pool): Extension<SqlitePool>,
    Form(vote): Form<UserStatementVote>,
) -> Html<String> {
    let user = ensure_auth(&cookies, &pool).await;
    let statement_id = vote.statement_id;

    let query = sqlx::query!(
        "INSERT INTO votes (user_id, statement_id, vote) VALUES (?, ?, ?)",
        user.id,
        statement_id,
        vote.vote
    )
    .execute(&pool)
    .await;
    query.expect("Database problem");

    index(Extension(pool)).await
}

#[derive(Serialize, sqlx::FromRow)]
struct User {
    id: i64,
    secret: String,
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

async fn ensure_auth(cookies: &Cookies, pool: &SqlitePool) -> User {
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

// Display one statement at random
async fn index(Extension(pool): Extension<SqlitePool>) -> Html<String> {
    let query =
        sqlx::query_as::<_, Statement>("SELECT id, text from statements ORDER BY RANDOM() LIMIT 1");
    let result = query.fetch_optional(&pool).await.expect("Must be valid");

    let template = IndexTemplate { statement: &result };

    Html(template.render().unwrap())
}
