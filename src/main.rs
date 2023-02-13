use dotenvy::dotenv;
use std::env;
use axum::{
    routing::{get},
    Router, Extension,
    http::StatusCode,
    Json, response::Html, extract::Path,
};

use std::net::SocketAddr;
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};

use serde::Serialize;
use askama::Template;

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
#[template(path = "hello.html")]

struct HelloTemplate<'a> {
    name: &'a str,
}

async fn hello(Path(name): Path<String>) -> Html<String> {
    let template = HelloTemplate { name: &name };
    Html(template.render().unwrap())
}

#[derive(Serialize)]
struct Statement {
    id: i64,
    text: String,
}

async fn root(Extension(pool):Extension<SqlitePool>) -> Result<Json<Statement>,StatusCode> {
    let row: Result<Statement, sqlx::Error> = sqlx::query_as!(Statement,"SELECT id, text from statements limit 1")
        .fetch_one(&pool).await;

    match row {
        Ok(row) => Ok(Json(row)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
