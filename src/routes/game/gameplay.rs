use std::sync::Arc;

use anyhow::{anyhow, bail};
use axum::extract::ws::Message;
use futures::{lock::Mutex, SinkExt, StreamExt};
use serde::{Deserialize, Serialize};

use super::{
    rules::{PieceColor, Position},
    OpenGame, SinkStream, SplitSink, SplitStream, WsMsg,
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

pub struct Gameplay {
    game: OpenGame,
    active_player: (PieceColor, SinkStream),
    passive_player: (PieceColor, SinkStream),
}

impl Gameplay {
    async fn ws_next(player: Arc<Mutex<SplitStream>>) -> anyhow::Result<GameMsgRecv> {
        let ws_message = player
            .lock()
            .await
            .next()
            .await
            .ok_or(anyhow!("No message received"))??;
        let Message::Text(message_text) = ws_message else {
            bail!("Incorrect WebSocket message type");
        };
        Ok(serde_json::from_str::<GameMsgRecv>(&message_text)?)
    }

    async fn ws_send(player: Arc<Mutex<SplitSink>>, msg: GameMessage) -> anyhow::Result<()> {
        player
            .lock()
            .await
            .send(Message::Text(serde_json::to_string(&WsMsg::Game(msg))?))
            .await
            .map_err(|err| err.into())
    }

    async fn ws_send_active(&mut self, msg: GameMessage) -> anyhow::Result<()> {
        Self::ws_send(self.active_player.1 .0.clone(), msg).await
    }

    async fn ws_send_passive(&mut self, msg: GameMessage) -> anyhow::Result<()> {
        Self::ws_send(self.passive_player.1 .0.clone(), msg).await
    }

    async fn ws_next_active(&mut self) -> anyhow::Result<GameMsgRecv> {
        Self::ws_next(self.active_player.1 .1.clone()).await
    }

    async fn ws_next_passive(&mut self) -> anyhow::Result<GameMsgRecv> {
        Self::ws_next(self.passive_player.1 .1.clone()).await
    }

    pub fn new(game: OpenGame) -> Self {
        Self {
            active_player: (PieceColor::White, game.user_stream.white_player.clone()),
            passive_player: (PieceColor::Black, game.user_stream.black_player.clone()),
            game,
        }
    }

    pub async fn run(&mut self) -> anyhow::Result<()> {
        // Wait for both players to acknowledge their involvement
        if !matches!(self.ws_next_active().await?, GameMsgRecv::Ack) {
            bail!("No white player ack");
        };
        if !matches!(self.ws_next_passive().await?, GameMsgRecv::Ack) {
            bail!("No black player ack");
        };
        self.ws_send_active(GameMessage::NewTurn(true)).await?;
        self.ws_send_passive(GameMessage::NewTurn(false)).await?;
        loop {
            let player_msg = loop {
                let player_msg = match self.ws_next_active().await {
                    Ok(msg) => msg,
                    Err(err) => {
                        self.ws_send_active(GameMessage::Error(format!("{:?}", err)))
                            .await?;
                        continue;
                    }
                };
                break player_msg;
            };
            match player_msg {
                GameMsgRecv::TurnEnd(piece_move) => {
                    if let Err(error) = self.handle_turn_end(piece_move).await {
                        println!("Error: {:?}", error);
                        self.ws_send_active(GameMessage::Error(format!("{:?}", error)))
                            .await?;
                        continue;
                    };
                }
                GameMsgRecv::Ack => {
                    continue;
                }
            };
            match self.handle_win().await {
                Ok(false) => {
                    self.switch_turns().await?;
                }
                Ok(true) => {
                    break;
                }
                Err(error) => {
                    println!("Error: {:?}", error);
                    self.ws_send_active(GameMessage::Error(format!("{:?}", error)))
                        .await?;
                }
            };
        }
        Ok(())
    }

    async fn handle_turn_end(&mut self, piece_move: ChessMove) -> anyhow::Result<()> {
        let chess_board_arc = self.game.chess_board.clone();
        let chess_piece = chess_board_arc
            .lock()
            .await
            .find_own_piece_at(&piece_move.position_from, self.active_player.0.clone())
            .ok_or(anyhow!("You don't have a piece at this position!"))?;
        chess_piece
            .lock()
            .await
            .move_piece_to(self.game.chess_board.clone(), &piece_move.position_to)
            .await?;
        self.ws_send_active(GameMessage::Notification("Moved correctly".into()))
            .await?;
        self.ws_send_passive(GameMessage::PawnMove(piece_move))
            .await?;
        Ok(())
    }

    async fn switch_turns(&mut self) -> anyhow::Result<()> {
        std::mem::swap(&mut self.active_player, &mut self.passive_player);
        self.ws_send_active(GameMessage::NewTurn(true)).await?;
        self.ws_send_passive(GameMessage::NewTurn(false)).await?;
        Ok(())
    }

    async fn handle_win(&mut self) -> anyhow::Result<bool> {
        let mut board_lock = self.game.chess_board.lock().await;
        let white_king = board_lock.find_king(PieceColor::White);
        let black_king = board_lock.find_king(PieceColor::Black);
        let winning_king = match (white_king, black_king) {
            (None, Some(_)) => Some(PieceColor::Black),
            (Some(_), None) => Some(PieceColor::White),
            (Some(_), Some(_)) => None,
            (None, None) => {
                bail!("There is no king on the field! The game encountered a critical error");
            }
        };
        if let Some(winning_color) = winning_king {
            let (winner, loser) = if self.active_player.0 == winning_color {
                (self.active_player.clone(), self.passive_player.clone())
            } else {
                (self.passive_player.clone(), self.active_player.clone())
            };
            Self::ws_send(winner.1 .0.clone(), GameMessage::GameEnd(true)).await?;
            Self::ws_send(loser.1 .0.clone(), GameMessage::GameEnd(false)).await?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}
