use anyhow::{anyhow, bail};
use axum::extract::ws::Message;
use futures::{lock::Mutex, SinkExt, StreamExt};
use serde::{Deserialize, Serialize};

use crate::routes::game::rules::PieceType;

use super::{
    rules::{ChessBoard, PieceColor, Position},
    OpenGame, WsMsg,
};

#[derive(Deserialize, Debug, Serialize, Clone)]
pub struct ChessMove {
    position_from: Position,
    position_to: Position,
}

#[derive(Deserialize, Debug)]
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
    PawnMove(ChessMove),
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
        passive_player
            .0
            .lock()
            .await
            .send(Message::Text(serde_json::to_string(&WsMsg::Game(
                GameMessage::NewTurn(false),
            ))?))
            .await?;
        let player_msg = loop {
            let ws_msg = active_player.1.lock().await.next().await;
            let Some(Ok(Message::Text(message))) = ws_msg else {
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
            let ws_msg_struct = serde_json::from_str::<GameMsgRecv>(&message);
            let Ok(player_msg) = ws_msg_struct else {
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
                let chess_board_arc = game.chess_board.clone();
                let mut chess_board_mutex = chess_board_arc.lock().await;
                let Some(chess_piece) = chess_board_mutex
                    .find_own_piece_at(&player_move.position_from, active_color.clone())
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
                drop(chess_board_mutex);
                if let Err(error) = chess_piece
                    .lock()
                    .await
                    .move_piece_to(game.chess_board.clone(), &player_move.position_to)
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
                println!("Sending info to the next player about the opponent move...");
                if let Err(err) = passive_player
                    .0
                    .lock()
                    .await
                    .send(Message::Text(serde_json::to_string(&WsMsg::Game(
                        GameMessage::PawnMove(player_move),
                    ))?))
                    .await
                {
                    println!("{:?}", err);
                };
                println!("Sent to passive player correctly...");
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
            .find(|piece| {
                piece.try_lock().unwrap().color == PieceColor::White
                    && piece.try_lock().unwrap().piece_type == PieceType::King
            })
            .cloned();
        let black_king = board_lock
            .pieces
            .iter()
            .find(|piece| {
                piece.try_lock().unwrap().color == PieceColor::Black
                    && piece.try_lock().unwrap().piece_type == PieceType::King
            })
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
