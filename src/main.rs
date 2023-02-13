use dotenvy::dotenv;
use std::env;
use tower_cookies::{Cookie, CookieManagerLayer, Cookies};
use axum::{
    Form,
    routing::{get, post},
    Router, Extension,
    http::StatusCode,
    Json, response::Html, extract::Path,
};

use std::net::SocketAddr;
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};

use serde::{Deserialize, Serialize};
use askama::{Template};
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;

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
        .route("/next", get(root))
        .route("/hello/:name", get(hello))
        .layer(Extension(pool))
        .layer(CookieManagerLayer::new())
        ;

    let addr = SocketAddr::from(([127, 0 , 0, 1], 8000));
    println!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();

}


#[derive(Template)]
#[template(path = "index.j2")]
struct IndexTemplate<'a> {
    statement: &'a Statement,
}

#[derive(Template)]
#[template(path = "hello.html")]
struct HelloTemplate<'a> {
    name: &'a str,
}

async fn hello(Path(name): Path<String>) -> Html<String> {
    let template = HelloTemplate { name: &name };
    Html(template.render().unwrap())
}

#[derive(Serialize, sqlx::FromRow)]
struct Statement {
    id: i64,
    text: String,
}

#[derive(Deserialize)]
struct UserStatementSelection {
    statement_id: i64,
    selection: String
}

async fn index_post(
    cookies: Cookies,
    Extension(pool):Extension<SqlitePool>,
    Form(selection): Form<UserStatementSelection>) -> Html<String>
{
    let user = ensure_auth(&cookies, &pool).await;
    let statement_id = selection.statement_id;
    let opinion = match selection.selection.as_str() {
        "y" => { 1 }
        "n" => { -1 }
        _ => { 0 }
    };

    let query = sqlx::query!(
        "INSERT INTO opinions (user_id, statement_id, opinion) VALUES (?, ?, ?)",
        user.id, statement_id, opinion)
        .execute(&pool).await;
    query.expect("Database problem");

    index(cookies, Extension(pool)).await
}

#[derive(Serialize, sqlx::FromRow)]
struct User {
    id: i64,
    secret: String,
}

async fn create_user(pool: &SqlitePool) -> User {
    let secret = generate_secret();
    let user = sqlx::query_as::<_, User>(
        "INSERT INTO users (secret) VALUES (?) RETURNING id, secret")
        .bind(secret)
        .fetch_one(pool).await.expect("Must be valid");

    user
}

async fn ensure_auth(cookies: &Cookies,
                     pool: &SqlitePool) -> User {
    if let Some(secret) = cookies.get("secret") {
        let secret = secret.value().to_string();
        let query = sqlx::query_as!(User, "SELECT id, secret from users WHERE secret = ?", secret);

        let result = query.fetch_optional(pool).await.expect("Must be valid");

        match result {
            Some(result) => {
                User {
                    id: result.id,
                    secret: secret.to_string(),
                }
            }
            None => {
                let user = create_user(pool).await;
                cookies.add(Cookie::new("secret", user.secret.to_owned()));

                user
            }
        }
    }
    else
    {
        let user = create_user(pool).await;
        cookies.add(Cookie::new("secret", user.secret.to_owned()));

        user
    }
}


// Display one statement at random
async fn index(
    cookies: Cookies,
    Extension(pool):Extension<SqlitePool>) -> Html<String> {

    let query = sqlx::query_as::<_, Statement>("SELECT id, text from statements ORDER BY RANDOM() LIMIT 1");
    let result = query.fetch_one(&pool).await.expect("Must be valid");

    let template = IndexTemplate {
        statement: &result
    };

    Html(template.render().unwrap())

}


async fn root(Extension(pool):Extension<SqlitePool>) -> Result<Json<Statement>,StatusCode> {
    let row: Result<Statement, sqlx::Error> = sqlx::query_as!(Statement,"SELECT id, text from statements limit 1")
        .fetch_one(&pool).await;

    match row {
        Ok(row) => Ok(Json(row)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
