use std::sync::Arc;

use anyhow::anyhow;
use axum::{
    debug_handler,
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    response::Response,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};

use crate::{
    routes::{
        game::{
            chessboard::ChessBoard, gameplay::Gameplay, piece::PieceColor, ws::GameWs,
            ws_messages::ServerMsg, OpenGame, PlayerStreams,
        },
        user::jwt::Claims,
    },
    GlobalState, ServerState,
};

use super::{
    matchmaking_state::{MatchmakingPlayer, UserQueue},
    Game,
};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) enum MatchmakingServerMsg {
    Searching,
    Success { color: PieceColor },
}

#[debug_handler(state=ServerState)]
pub async fn route_handler(
    ws: WebSocketUpgrade,
    State(queue_state): State<UserQueue>,
    State(global_state): State<GlobalState>,
) -> Response {
    ws.on_upgrade(|socket: WebSocket| handle_ws(global_state, socket, queue_state))
}

pub async fn handle_ws(
    GlobalState { db_pool }: GlobalState,
    socket: WebSocket,
    user_queue: UserQueue,
) {
    let ws = GameWs::new(socket);

    // Await authentication
    let Ok(Message::Text(jwt_str)) = ws.get().await else {
        return;
    };
    // Check if the claims are correct
    let claims = Claims::try_from(jwt_str);
    let claims = match claims {
        Ok(claims) => claims,
        Err(error_val) => {
            println!("Returning, invalid JWT error: {}", error_val);
            return;
        }
    };

    // create an on_message handler
    let echo_task = Arc::new(tokio::spawn(ws_matchmaking(
        ws.clone(),
        user_queue.clone(),
        claims.sub,
    )));

    // Insert this user into the queue with send, receive, and on_message handler
    user_queue.get().await.insert(
        claims.sub,
        MatchmakingPlayer::new(ws.clone(), echo_task.clone()),
    );
    // check if we have 2 players. if so, start game.
    let users_in_queue = user_queue
        .get()
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
            chess_board,
            user_stream: PlayerStreams {
                white_player: opponent_state.ws.clone(),
                black_player: ws.clone(),
            },
        };
        // Inform the clients of the new game
        // Send the JSON as text message to both players
        let Ok(_) = [
            ws.send(Message::Text(
                serde_json::to_string(&ServerMsg::Matchmaking(MatchmakingServerMsg::Success {
                    color: PieceColor::Black,
                }))
                .unwrap(),
            ))
            .await,
            opponent_state
                .ws
                .send(Message::Text(
                    serde_json::to_string(&ServerMsg::Matchmaking(MatchmakingServerMsg::Success {
                        color: PieceColor::White,
                    }))
                    .unwrap(),
                ))
                .await,
        ]
        .into_iter()
        .collect::<Result<Vec<()>, _>>() else {
            return;
        };

        // Stop the echo services
        opponent_state.echo_task.abort();
        echo_task.abort();

        // Start the game :D
        let _ = Gameplay::new(open_game).run().await;
    }
}

async fn ws_matchmaking(ws: GameWs, user_queue: UserQueue, user_id: i32) {
    while let Ok(message) = ws.get().await {
        if let Message::Close(_) = message {
            user_queue.get().await.remove(&user_id);
            return;
        }
        let msg = MatchmakingServerMsg::Searching;
        let msg = ServerMsg::Matchmaking(msg);
        let msg = serde_json::to_string(&msg);
        let msg = msg.unwrap_or_default();
        let msg = Message::Text(msg);
        ws.send(msg).await.unwrap();
    }
}

async fn create_game(
    db_pool: &Pool<Postgres>,
    username_black: i32,
    username_white: i32,
) -> anyhow::Result<Game> {
    sqlx::query_as!(
        Game,
        "INSERT INTO game (started_at, player_black, player_white) VALUES ($1, $2, $3) RETURNING *",
        Utc::now().naive_utc(),
        username_black,
        username_white
    )
    .fetch_one(db_pool)
    .await
    .map_err(|err| anyhow!(err))
}
