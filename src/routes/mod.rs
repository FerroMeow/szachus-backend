use core::str;

use axum::{
    extract::State,
    middleware,
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use tower_http::services::ServeDir;

use crate::{templates::get_template, ServerState};

pub mod game;
mod user;

pub fn app_routes() -> Router<ServerState> {
    Router::new()
        .nest("/game", game::routes())
        .layer(middleware::from_extractor::<user::jwt::Claims>())
        .route("/", get(start_page))
        .nest("/user", user::authentication_routes())
        .fallback_service(ServeDir::new("public").not_found_service(get(no_route_error)))
}

#[axum::debug_handler]
async fn start_page(State(ServerState { handlebars, .. }): State<ServerState>) -> Html<String> {
    Html(get_template(&handlebars, "index", &0).await.unwrap())
}

async fn no_route_error() -> impl IntoResponse {
    let Ok(page_404) = tokio::fs::read("public/404.html").await else {
        return Html("".to_string());
    };
    Html(str::from_utf8(&page_404[..]).unwrap().to_string())
}
