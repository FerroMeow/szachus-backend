use axum::{
    debug_handler,
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    response::Response,
};
use db::{create_game, remove_game, Game};
use matchmaking_state::{MatchmakingPlayer, UserQueue};
use sqlx::{Pool, Postgres};
use ws_message::MatchmakingServerMsg;

use crate::{routes::user::jwt::Claims, GlobalState, ServerState};

use super::{
    gameplay::Gameplay, opponent_pair::OpponentPair, piece_color::PieceColor, ws::GameWs,
    ws_messages::ServerMsg,
};

pub mod db;
pub mod matchmaking_state;
pub mod ws_message;
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
        Err(_) => {
            let _ = ws
                .send_as_text(&ServerMsg::Matchmaking(MatchmakingServerMsg::Error(
                    "Invalid JWT!".into(),
                )))
                .await;
            return;
        }
    };

    let user_in_queue = user_queue
        .state
        .lock()
        .await
        .make_contiguous()
        .iter()
        .any(|user| user.id == claims.sub);
    if user_in_queue {
        ws.send_as_text(&MatchmakingServerMsg::Error(
            "User already in the queue".into(),
        ))
        .await
        .unwrap();
        return;
    }

    // Create a service for matchmaking player
    let echo_task = tokio::spawn(ws_matchmaking(ws.clone(), user_queue.clone(), claims.sub));
    let matchmaking_player = MatchmakingPlayer::new(claims.sub, ws, echo_task);
    // check if we have 2 players. if so, start game.
    let matchmaking_opponent = match user_queue.pop().await {
        // Push user into queue and return. We do not need to continue the function.
        None => {
            matchmaking_player
                .ws
                .send_as_text(&MatchmakingServerMsg::Searching)
                .await
                .unwrap();
            user_queue.push(matchmaking_player).await;
            return;
        }
        // Opponent for current player found!
        Some(player) => player,
    };
    // Insert info about the new game into the database
    let Ok(game_data) = create_game(&db_pool, matchmaking_player.id, matchmaking_opponent.id).await
    else {
        return;
    };
    // Inform the players of the new game
    let _ = matchmaking_player
        .ws
        .send_as_text(&ServerMsg::Matchmaking(MatchmakingServerMsg::Success {
            color: PieceColor::Black,
        }))
        .await;
    let _ = matchmaking_opponent
        .ws
        .send_as_text(&ServerMsg::Matchmaking(MatchmakingServerMsg::Success {
            color: PieceColor::White,
        }))
        .await;

    // Stop the echo services
    matchmaking_player.echo.abort();
    matchmaking_opponent.echo.abort();
    let opponent_pair = OpponentPair::new(matchmaking_opponent, matchmaking_player);
    tokio::spawn(game_session(db_pool.clone(), game_data, opponent_pair));
}

async fn game_session(db_pool: Pool<Postgres>, game_data: Game, opponent_pair: OpponentPair) {
    let mut open_game = Gameplay::new(db_pool.clone(), game_data, opponent_pair);
    // Start the game :D
    let game_result = open_game.run().await;
    // Check for errors
    if let Err(error) = game_result {
        // Game has encountered an error. Notify the active players.
        let error = ServerMsg::Matchmaking(MatchmakingServerMsg::GameDropped(error.to_string()));
        // This operation will probably foil for one of them, so we ignore the errors, as this is an error handler.
        let _ = open_game.players.white_player.ws.send_as_text(&error).await;
        let _ = open_game.players.black_player.ws.send_as_text(&error).await;
        // We still want to panic on database errors though
        remove_game(&db_pool, open_game.game_data).await.unwrap();
    };
}

async fn ws_matchmaking(ws: GameWs, user_queue: UserQueue, user_id: i32) {
    while let Ok(message) = ws.get().await {
        if let Message::Close(_) = message {
            let user_index = user_queue
                .state
                .lock()
                .await
                .iter()
                .position(|player| player.id == user_id);
            let Some(user_index) = user_index else {
                continue;
            };
            user_queue.state.lock().await.remove(user_index);
            return;
        }
    }
}
