use std::sync::Arc;

use axum::{
    debug_handler,
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    response::Response,
};

use chrono::prelude::*;
use futures::{future::join_all, lock::Mutex, SinkExt, StreamExt};
use serde::Serialize;
use sqlx::{Pool, Postgres};

use crate::{
    error,
    routes::user::{self, jwt::Claims},
    GlobalState, ServerState,
};

use super::{gameplay::gameplay_loop, MatchmakingState, OpenGame};

#[derive(Serialize, Clone)]
pub struct Game {
    id: i32,
    started_at: NaiveDateTime,
    ended_at: Option<NaiveDateTime>,
    player_black: i32,
    player_white: i32,
}

#[debug_handler(state=ServerState)]
pub async fn route_handler(
    ws: WebSocketUpgrade,
    State(queue_state): State<MatchmakingState>,
    State(global_state): State<GlobalState>,
    claims: user::jwt::Claims,
) -> Response {
    ws.on_upgrade(|socket: WebSocket| handle_ws(global_state, claims, socket, queue_state))
}

#[derive(Serialize)]
enum MatchmakingResponse {
    Searching,
    Success(Game),
}

async fn handle_ws(
    GlobalState { db_pool }: GlobalState,
    claims: Claims,
    socket: WebSocket,
    MatchmakingState(user_queue): MatchmakingState,
) {
    let (tx, rx) = socket.split();

    let (tx, rx) = (Arc::new(Mutex::new(tx)), Arc::new(Mutex::new(rx)));
    let users_in_queue = user_queue
        .lock()
        .await
        .iter()
        .find(|(id, _)| **id != claims.sub)
        .map(|(id, tx)| (*id, tx.clone()));
    let (echo_tx, echo_rx) = (tx.clone(), rx.clone());
    let echo_task = Arc::new(tokio::spawn(async move {
        while let Some(Ok(_)) = echo_rx.clone().lock().await.next().await {
            let _ = echo_tx
                .clone()
                .lock()
                .await
                .send(Message::Text(
                    serde_json::to_string(&MatchmakingResponse::Searching)
                        .unwrap_or("".to_string()),
                ))
                .await;
            println!("Response sent");
        }
    }));
    if let Some((opponent_id, opponent_state)) = users_in_queue {
        let Ok(game) = create_game(&db_pool, claims.sub, opponent_id).await else {
            return;
        };
        let Ok(game_json) = serde_json::to_string(&MatchmakingResponse::Success(game.clone()))
        else {
            return;
        };
        let Ok(_) = join_all([tx.clone(), opponent_state.0.clone()].map(|tx| {
            let game_json = game_json.clone();
            async move {
                tx.clone()
                    .lock()
                    .await
                    .send(Message::Text(game_json))
                    .await
                    .map_err(|err| {
                        println!("{:?}", err);
                        err
                    })
            }
        }))
        .await
        .into_iter()
        .collect::<Result<Vec<()>, axum::Error>>() else {
            return;
        };
        let open_game = OpenGame {
            game_data: game,
            user_stream: (
                (tx.clone(), rx.clone()),
                (opponent_state.0.clone(), opponent_state.1),
            ),
        };
        let game_echo_task = echo_task.clone();
        let _ = tokio::spawn(async move {
            opponent_state.2.abort();
            game_echo_task.abort();
            gameplay_loop(open_game).await
        })
        .await;
    }
    user_queue
        .clone()
        .lock()
        .await
        .insert(claims.sub, (tx.clone(), rx.clone(), echo_task.clone()));
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
    .map_err(|err| {
        println!("{:?}", &err);
        err.into()
    })
}
