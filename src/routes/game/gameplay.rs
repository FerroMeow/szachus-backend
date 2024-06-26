use anyhow::bail;
use axum::extract::ws::Message;
use futures::{SinkExt, StreamExt};
use serde::Deserialize;

use super::{
    rules::{PieceColor, Position},
    OpenGame,
};

#[derive(Deserialize)]
pub struct ChessMove {
    position_from: Position,
    position_to: Position,
}

pub async fn gameplay_loop(game: OpenGame) -> anyhow::Result<()> {
    let mut is_firsts_turn = true;
    loop {
        let active_color = if is_firsts_turn {
            PieceColor::White
        } else {
            PieceColor::Black
        };
        let (active_player, passive_player) = match is_firsts_turn {
            true => (&game.user_stream.0, &game.user_stream.1),
            false => (&game.user_stream.1, &game.user_stream.0),
        };
        active_player
            .0
            .lock()
            .await
            .send(Message::Text(String::from("It's your turn")))
            .await?;
        passive_player
            .0
            .lock()
            .await
            .send(Message::Text(String::from(
                "It's not your turn, wait for your opponent's action",
            )))
            .await?;

        loop {
            let Some(Ok(Message::Text(message))) = active_player.1.lock().await.next().await else {
                active_player
                    .0
                    .lock()
                    .await
                    .send(Message::Text(String::from(
                        "The message is not a valid string!",
                    )))
                    .await?;
                bail!("Bad message error");
            };
            let Ok(player_move) = serde_json::from_str::<ChessMove>(&message) else {
                active_player
                    .0
                    .lock()
                    .await
                    .send(Message::Text(String::from("Message is not a valid move!")))
                    .await?;
                continue;
            };
            let Some(mut chess_piece) = game
                .chess_board
                .clone()
                .lock()
                .await
                .find_own_piece_at(&player_move.position_from, active_color.clone())
                .cloned()
            else {
                active_player
                    .0
                    .lock()
                    .await
                    .send(Message::Text(
                        "You don't have a piece at  this position".to_string(),
                    ))
                    .await?;
                continue;
            };

            if let Err(error) = chess_piece
                .move_piece_to(game.chess_board.clone(), player_move.position_to)
                .await
            {
                active_player
                    .0
                    .lock()
                    .await
                    .send(Message::Text(error.to_string()))
                    .await?;
                continue;
            };
            active_player
                .0
                .lock()
                .await
                .send(Message::Text(String::from("Moved correctly.")))
                .await?;
            break;
        }
        is_firsts_turn = !is_firsts_turn;
    }
}
