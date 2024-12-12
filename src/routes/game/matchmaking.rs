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
use futures::{lock::Mutex, SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};

use crate::{
    error,
    routes::{game::WsMsg, user::jwt::Claims},
    GlobalState, ServerState,
};

use super::{
    gameplay,
    rules::{ChessBoard, PieceColor},
    MatchmakingState, OpenGame, PlayerStreams,
};

#[derive(Serialize, Deserialize, Clone)]
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
) -> Response {
    ws.on_upgrade(|socket: WebSocket| handle_ws(global_state, socket, queue_state))
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) enum MatchmakingResponse {
    Searching,
    Success { color: PieceColor },
}

async fn handle_ws(
    GlobalState { db_pool }: GlobalState,
    socket: WebSocket,
    MatchmakingState(user_queue): MatchmakingState,
) {
    let (tx, rx) = socket.split();
    let (tx, rx) = (Arc::new(Mutex::new(tx)), Arc::new(Mutex::new(rx)));

    // Await authentication
    let Some(Ok(Message::Text(jwt_str))) = rx.lock().await.next().await else {
        return;
    };
    // Check if the claims are correct
    let claims = match Claims::try_from(jwt_str) {
        Ok(claims) => claims,
        Err(error_val) => {
            println!("Returning, invalid JWT error: {}", error_val);
            return;
        }
    };

    // create an on_message handler
    let (echo_tx, echo_rx) = (tx.clone(), rx.clone());
    let echo_user_queue = user_queue.clone();
    let echo_task = Arc::new(tokio::spawn(async move {
        while let Some(Ok(message)) = echo_rx.clone().lock().await.next().await {
            if let Message::Close(_) = message {
                echo_user_queue.lock().await.remove(&claims.sub);
                return;
            }
            let _ = echo_tx
                .clone()
                .lock()
                .await
                .send(Message::Text(
                    serde_json::to_string(&WsMsg::Matchmaking(MatchmakingResponse::Searching))
                        .unwrap_or("".to_string()),
                ))
                .await;
        }
    }));

    // Insert this user into the queue with send, receive, and on_message handler
    user_queue
        .clone()
        .lock()
        .await
        .insert(claims.sub, (tx.clone(), rx.clone(), echo_task.clone()));
    // check if we have 2 players. if so, start game.
    let users_in_queue = user_queue
        .lock()
        .await
        .iter()
        .find(|(id, _)| **id != claims.sub)
        .map(|(id, tx)| (*id, tx.clone()));
    if let Some((opponent_id, opponent_state)) = users_in_queue {
        // Found 2 users, generating new game
        let Ok(game_data) = create_game(&db_pool, claims.sub, opponent_id).await else {
            return;
        };
        let Ok(chess_board) = ChessBoard::new() else {
            return;
        };
        let open_game = OpenGame {
            game_data: game_data.clone(),
            chess_board: Arc::new(Mutex::new(chess_board)),
            user_stream: PlayerStreams {
                white_player: (opponent_state.0.clone(), opponent_state.1),
                black_player: (tx.clone(), rx.clone()),
            },
        };
        // Inform the clients of the new game
        // Send the JSON as text message to both players
        let Ok(_) = [
            tx.clone()
                .lock()
                .await
                .send(Message::Text(
                    serde_json::to_string(&WsMsg::Matchmaking(MatchmakingResponse::Success {
                        color: PieceColor::Black,
                    }))
                    .unwrap(),
                ))
                .await,
            opponent_state
                .0
                .lock()
                .await
                .send(Message::Text(
                    serde_json::to_string(&WsMsg::Matchmaking(MatchmakingResponse::Success {
                        color: PieceColor::White,
                    }))
                    .unwrap(),
                ))
                .await,
        ]
        .into_iter()
        .collect::<Result<Vec<()>, axum::Error>>() else {
            return;
        };
        // Start the game :D
        let _ = tokio::spawn(async move {
            // Stop the echo services
            opponent_state.2.abort();
            echo_task.abort();
            gameplay::Gameplay::new(open_game).run().await
        })
        .await;
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
    .map_err(|err| {
        println!("{:?}", &err);
        err.into()
    })
}
