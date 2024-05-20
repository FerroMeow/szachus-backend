use axum::{
    extract::{ws::WebSocket, WebSocketUpgrade},
    response::IntoResponse,
};

pub async fn ws_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(handle_ws)
}

async fn handle_ws(mut socket: WebSocket) {}
