use anyhow::{anyhow, bail};
use axum::extract::ws::Message;
use futures::{lock::Mutex, SinkExt, StreamExt};
use serde::{Deserialize, Serialize};

use crate::routes::game::rules::PieceType;

use super::{
    rules::{ChessBoard, PieceColor, Position},
    OpenGame, WsMsg,
};

#[derive(Deserialize)]
pub struct ChessMove {
    position_from: Position,
    position_to: Position,
}

#[derive(Deserialize)]
pub(crate) enum GameMsgRecv {
    TurnEnd(ChessMove),
    Ack,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) enum GameMessage {
    NewTurn(bool),
    Error(String),
    Notification(String),
    GameEnd(bool),
}

pub async fn gameplay_loop(game: OpenGame) -> anyhow::Result<()> {
    // Wait for both players to acknowledge their involvement
    let white_ack = &game
        .user_stream
        .white_player
        .1
        .lock()
        .await
        .next()
        .await
        .and_then(|v| {
            v.ok()
                .and_then(|msg| match msg {
                    Message::Text(text) => Some(text),
                    _ => None,
                })
                .and_then(|text| {
                    serde_json::from_str::<GameMsgRecv>(&text)
                        .ok()
                        .and_then(|msg| match msg {
                            GameMsgRecv::Ack => Some(()),
                            _ => None,
                        })
                })
        });
    let black_ack = &game
        .user_stream
        .black_player
        .1
        .lock()
        .await
        .next()
        .await
        .and_then(|v| {
            v.ok()
                .and_then(|msg| match msg {
                    Message::Text(text) => Some(text),
                    _ => None,
                })
                .and_then(|text| {
                    serde_json::from_str::<GameMsgRecv>(&text)
                        .ok()
                        .and_then(|msg| match msg {
                            GameMsgRecv::Ack => Some(()),
                            _ => None,
                        })
                })
        });
    if white_ack.is_none() && black_ack.is_none() {
        anyhow::bail!("Not received ack!");
    }
    let mut is_firsts_turn = true;
    loop {
        println!("Is first turn: {is_firsts_turn}");
        let (active_color, active_player, passive_player) = match is_firsts_turn {
            true => (
                PieceColor::White,
                &game.user_stream.white_player,
                &game.user_stream.black_player,
            ),
            false => (
                PieceColor::Black,
                &game.user_stream.black_player,
                &game.user_stream.white_player,
            ),
        };
        active_player
            .0
            .lock()
            .await
            .send(Message::Text(serde_json::to_string(&WsMsg::Game(
                GameMessage::NewTurn(true),
            ))?))
            .await?;
        println!("Sent the active turn to the player");
        passive_player
            .0
            .lock()
            .await
            .send(Message::Text(serde_json::to_string(&WsMsg::Game(
                GameMessage::NewTurn(false),
            ))?))
            .await?;
        println!("Sent the passive turn to the player");
        let player_msg = loop {
            let Some(Ok(Message::Text(message))) = active_player.1.lock().await.next().await else {
                active_player
                    .0
                    .lock()
                    .await
                    .send(Message::Text(serde_json::to_string(&WsMsg::Game(
                        GameMessage::Error("The message is not a valid string!".into()),
                    ))?))
                    .await?;
                bail!("Bad message error");
            };
            let Ok(player_msg) = serde_json::from_str::<GameMsgRecv>(&message) else {
                active_player
                    .0
                    .lock()
                    .await
                    .send(Message::Text(serde_json::to_string(&WsMsg::Game(
                        GameMessage::Error("The message is not a valid move!".into()),
                    ))?))
                    .await?;
                continue;
            };
            break player_msg;
        };
        match player_msg {
            GameMsgRecv::TurnEnd(player_move) => {
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
                        .send(Message::Text(serde_json::to_string(&WsMsg::Game(
                            GameMessage::Error("You don't have a piece at this position!".into()),
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
                        .send(Message::Text(serde_json::to_string(&WsMsg::Game(
                            GameMessage::Error(error.to_string()),
                        ))?))
                        .await?;
                    continue;
                };
                active_player
                    .0
                    .lock()
                    .await
                    .send(Message::Text(serde_json::to_string(&WsMsg::Game(
                        GameMessage::Notification("Moved correctly".into()),
                    ))?))
                    .await?;
            }
            GameMsgRecv::Ack => (),
        };
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
                .send(Message::Text(serde_json::to_string(&WsMsg::Game(
                    GameMessage::GameEnd(true),
                ))?))
                .await?;
            loser
                .0
                .lock()
                .await
                .send(Message::Text(serde_json::to_string(&WsMsg::Game(
                    GameMessage::GameEnd(false),
                ))?))
                .await?;
            println!("Win!");
            return Ok(());
        };
        is_firsts_turn = !is_firsts_turn;
        println!("Changed the is_firsts_turn!");
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
