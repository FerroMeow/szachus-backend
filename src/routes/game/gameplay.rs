use anyhow::{anyhow, bail};
use axum::extract::ws::Message;
use futures::{lock::Mutex, SinkExt, StreamExt};
use serde::{Deserialize, Serialize};

use crate::routes::game::rules::PieceType;

use super::{
    rules::{ChessBoard, PieceColor, Position},
    OpenGame,
};

#[derive(Deserialize)]
pub struct ChessMove {
    position_from: Position,
    position_to: Position,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum GameMessage<'a> {
    NewTurn(bool),
    Error(&'a str),
    Notification(&'a str),
    GameEnd(bool),
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
            .send(Message::Text(serde_json::to_string(
                &GameMessage::NewTurn(true),
            )?))
            .await?;
        passive_player
            .0
            .lock()
            .await
            .send(Message::Text(serde_json::to_string(
                &GameMessage::NewTurn(false),
            )?))
            .await?;

        loop {
            let Some(Ok(Message::Text(message))) = active_player.1.lock().await.next().await else {
                active_player
                    .0
                    .lock()
                    .await
                    .send(Message::Text(serde_json::to_string(&GameMessage::Error(
                        "The message is not a valid string!",
                    ))?))
                    .await?;
                bail!("Bad message error");
            };
            let Ok(player_move) = serde_json::from_str::<ChessMove>(&message) else {
                active_player
                    .0
                    .lock()
                    .await
                    .send(Message::Text(serde_json::to_string(&GameMessage::Error(
                        "The message is not a valid move!",
                    ))?))
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
                    .send(Message::Text(serde_json::to_string(&GameMessage::Error(
                        "You don't have a piece at this position!",
                    ))?))
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
                    .send(Message::Text(serde_json::to_string(&GameMessage::Error(
                        &error.to_string(),
                    ))?))
                    .await?;
                continue;
            };
            active_player
                .0
                .lock()
                .await
                .send(Message::Text(serde_json::to_string(
                    &GameMessage::Notification("Moved correctly"),
                )?))
                .await?;
            break;
        }
        if let Some(winning_color) = check_win_condition(&game.chess_board).await? {
            let (winner, loser) = if active_color == winning_color {
                (active_player, passive_player)
            } else {
                (passive_player, active_player)
            };
            winner
                .0
                .lock()
                .await
                .send(Message::Text(serde_json::to_string(
                    &GameMessage::GameEnd(true),
                )?))
                .await?;
            loser
                .0
                .lock()
                .await
                .send(Message::Text(serde_json::to_string(
                    &GameMessage::GameEnd(false),
                )?))
                .await?;
            return Ok(());
        };
        is_firsts_turn = !is_firsts_turn;
    }
}

async fn check_win_condition(
    chess_board: &Mutex<ChessBoard>,
) -> anyhow::Result<Option<PieceColor>> {
    let (white_king, black_king) = {
        let board_lock = chess_board.lock().await;
        let white_king = board_lock
            .pieces
            .iter()
            .find(|piece| piece.color == PieceColor::White && piece.piece_type == PieceType::King)
            .cloned();
        let black_king = board_lock
            .pieces
            .iter()
            .find(|piece| piece.color == PieceColor::Black && piece.piece_type == PieceType::King)
            .cloned();
        (white_king, black_king)
    };

    match (white_king, black_king) {
        (None, Some(_)) => Ok(Some(PieceColor::Black)),
        (Some(_), None) => Ok(Some(PieceColor::White)),
        (Some(_), Some(_)) => Ok(None),
        (None, None) => Err(anyhow!(
            "There is no king on the field! The game encountered a critical error"
        )),
    }
}
