use std::collections::VecDeque;

use axum::{
    extract::{
        ws::{Message, WebSocket},
        FromRef, State, WebSocketUpgrade,
    },
    response::Response,
    Extension,
};

use chrono::prelude::*;
use serde::Serialize;
use sqlx::{Pool, Postgres};

use crate::{
    error,
    routes::authentication::{self, jwt::Claims},
    ServerState,
};

#[derive(Default, Clone)]
pub struct MatchmakingState(VecDeque<i32>);

impl FromRef<ServerState> for MatchmakingState {
    fn from_ref(input: &ServerState) -> Self {
        MatchmakingState(input.user_queue.clone())
    }
}

#[derive(Serialize)]
pub struct Game {
    id: i32,
    started_at: NaiveDateTime,
    ended_at: NaiveDateTime,
    player_black: i32,
    player_white: i32,
}

pub async fn route_handler(
    ws: WebSocketUpgrade,
    State(queue_state): State<MatchmakingState>,
    Extension(claims): Extension<authentication::jwt::Claims>,
    State(server_state): State<ServerState>,
) -> Response {
    ws.on_upgrade(|socket| handle_ws(server_state, socket, claims))
}

async fn handle_ws(
    ServerState {
        db_pool,
        mut user_queue,
    }: ServerState,
    mut socket: WebSocket,
    claims: Claims,
) {
    while let Some(msg) = socket.recv().await {
        if msg.is_err() {
            return;
        };
        let user_id = claims.sub;
        if user_queue.contains(&user_id) {
            continue;
        }
        let opponent_id = user_queue.pop_front();
        if opponent_id.is_none() {
            continue;
        }
        let game = create_game(&db_pool, user_id, opponent_id.unwrap()).await;
        let game = if let Ok(game) = game {
            game
        } else {
            return;
        };
        let game_json = if let Ok(game_json) = serde_json::to_string(&game) {
            game_json
        } else {
            return;
        };
        if (socket.send(Message::Text(game_json)).await).is_err() {
            return;
        };
    }
}

async fn create_game(
    db_pool: &Pool<Postgres>,
    username_black: i32,
    username_white: i32,
) -> error::Result<Game> {
    sqlx::query_as!(
        Game,
        "INSERT INTO game (started_at, player_black, player_white) VALUES ($1, $2, $3) RETURNING *",
        Utc::now().naive_utc(),
        username_black,
        username_white
    )
    .fetch_one(db_pool)
    .await
    .map_err(|err| err.into())
}
