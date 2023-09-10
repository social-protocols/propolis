use std::net::SocketAddr;

use crate::api;
use crate::http_static::static_handler;
use crate::pages;
use crate::pages::new_statement::create_statement;
use crate::pages::new_statement::new_statement_completions;
use crate::pages::subscribe::subscribe;
use crate::pages::user::user_page;
use crate::pages::vote::vote;
use crate::pages::vote::vote_post;
use anyhow::Result;
use axum::routing::post;
use axum::Extension;
use axum::{routing::get, Router};
use http::StatusCode;
use pages::index::{index, search_results};
use pages::merge::{merge, merge_post};
use pages::new_statement::new_statement;
use pages::options::options;
use pages::statement::statement_page;
use pages::subscriptions::subscriptions;
use sqlx::SqlitePool;
use tower_cookies::CookieManagerLayer;
use tower_http::compression::CompressionLayer;
use tower_http::trace::TraceLayer;
use tracing::info;

pub async fn start_http_server(sqlite_pool: SqlitePool) -> Result<()> {
    let mut app = Router::new();

    app = app
        .route("/", get(index))
        .route("/search", post(search_results))
        .route("/vote", get(vote))
        .route("/vote", post(vote_post))
        .route("/subscribe", post(subscribe))
        .route("/user", get(user_page))
        .route("/statement/:id", get(statement_page))
        .route("/merge/:secret", get(merge))
        .route("/merge/:secret", post(merge_post))
        .route("/new", get(new_statement))
        .route("/new/completions", post(new_statement_completions))
        .route("/create", post(create_statement))
        .route("/options", get(options))
        .route("/subscriptions", get(subscriptions));

    #[cfg(feature = "with_predictions")]
    {
        app = app.route(
            "/prediction/:id",
            get(crate::pages::prediction::prediction_page),
        );
    }

    let apiv0 = Router::new()
        .route("/user/create", post(api::create_user))
        .route("/next_statement", get(api::next_statement))
        .route("/statement/:id/vote", post(api::statement_vote))
        .layer(Extension(sqlite_pool.clone()));

    app = app
        .route("/healthy", get(handler_healthy))
        .route("/*file", get(static_handler))
        .layer(TraceLayer::new_for_http())
        .layer(Extension(sqlite_pool.to_owned()))
        .layer(CookieManagerLayer::new())
        .layer(CompressionLayer::new())
        .fallback_service(get(not_found));

    let addr = SocketAddr::from(([0, 0, 0, 0], 8000));
    info!("Http server listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.nest("/api/v0", apiv0).into_make_service())
        .await?;

    Ok(())
}

async fn handler_healthy() -> StatusCode {
    StatusCode::OK
}

async fn not_found() -> StatusCode {
    StatusCode::NOT_FOUND
}
