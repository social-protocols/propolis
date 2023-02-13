use dotenvy::dotenv;
use std::env;
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
        .layer(Extension(pool));

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
    Extension(pool):Extension<SqlitePool>,
    Form(selection): Form<UserStatementSelection>) -> Html<String>
{
    let user_id = 1;
    let statement_id = 1;
    let opinion = match selection.selection.as_str() {
        "y" => { 1 }
        "n" => { -1 }
        _ => { 0 }
    };

    let query = sqlx::query!(
        "INSERT INTO opinions (user_id, statement_id, opinion) VALUES (?, ?, ?)",
        user_id, statement_id, opinion)
        .execute(&pool).await;
    query.expect("Database problem");

    index(Extension(pool)).await
}


// Display one statement at random
async fn index(Extension(pool):Extension<SqlitePool>) -> Html<String> {

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
