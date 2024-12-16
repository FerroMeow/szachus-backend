use anyhow::bail;
use axum::extract::ws::Message;
use chessboard::ChessBoard;
use ws_message::GameServerMsg;

use super::matchmaking::db::Game;
use super::opponent_pair::OpponentPair;

use super::piece_color::PieceColor;
use super::ws::GameWs;
use super::ws_messages::{ChessMove, GameClientMsg, ServerMsg};

pub mod chessboard;
pub mod piece;
pub mod position;
pub mod ws_message;

#[derive(Debug)]
pub struct Gameplay {
    game_data: Game,
    chess_board: ChessBoard,
    players: OpponentPair,
}

impl Gameplay {
    async fn ws_next(player: &GameWs) -> anyhow::Result<GameClientMsg> {
        let ws_message = player.get().await?;
        let Message::Text(message_text) = ws_message else {
            bail!("Incorrect WebSocket message type");
        };
        Ok(serde_json::from_str::<GameClientMsg>(&message_text)?)
    }

    async fn ws_send(player: &GameWs, msg: GameServerMsg) -> anyhow::Result<()> {
        player
            .send(Message::Text(serde_json::to_string(&ServerMsg::Game(msg))?))
            .await
    }

    pub fn new(game_data: Game, players: OpponentPair) -> Self {
        Self {
            game_data,
            chess_board: ChessBoard::new(),
            players,
        }
    }

    async fn ws_send_active(&mut self, msg: GameServerMsg) -> anyhow::Result<()> {
        Self::ws_send(self.players.get_active(), msg).await
    }

    async fn ws_send_passive(&mut self, msg: GameServerMsg) -> anyhow::Result<()> {
        Self::ws_send(self.players.get_passive(), msg).await
    }

    async fn ws_next_active(&mut self) -> anyhow::Result<GameClientMsg> {
        Self::ws_next(self.players.get_active()).await
    }

    async fn ws_next_passive(&mut self) -> anyhow::Result<GameClientMsg> {
        Self::ws_next(self.players.get_passive()).await
    }

    async fn handle_turn_end(&mut self, piece_move: ChessMove) -> anyhow::Result<()> {
        let removed_piece_maybe = self
            .chess_board
            .move_piece(
                &self.players.current_player_color,
                piece_move.position_from,
                piece_move.position_to,
            )
            .await?;
        let removed_piece_to = removed_piece_maybe.map(|lock| (lock.color, piece_move.position_to));
        self.ws_send_active(GameServerMsg::MovedCorrectly(removed_piece_to))
            .await?;
        self.ws_send_passive(GameServerMsg::PawnMove(piece_move, removed_piece_to))
            .await?;
        Ok(())
    }

    async fn switch_turns(&mut self) -> anyhow::Result<()> {
        self.players.switch_active();
        self.ws_send_active(GameServerMsg::NewTurn(true)).await?;
        self.ws_send_passive(GameServerMsg::NewTurn(false)).await?;
        Ok(())
    }

    async fn handle_win(&mut self) -> anyhow::Result<bool> {
        let white_king = self.chess_board.find_king(PieceColor::White);
        let black_king = self.chess_board.find_king(PieceColor::Black);
        let winning_king = match (white_king, black_king) {
            (None, Some(_)) => Some(PieceColor::Black),
            (Some(_), None) => Some(PieceColor::White),
            (Some(_), Some(_)) => None,
            (None, None) => {
                bail!("There is no king on the field! The game encountered a critical error");
            }
        };
        if let Some(winning_color) = winning_king {
            let (winner, loser) = if self.players.current_player_color == winning_color {
                (self.players.get_active(), self.players.get_passive())
            } else {
                (self.players.get_passive(), self.players.get_active())
            };
            Self::ws_send(winner, GameServerMsg::GameEnd(true)).await?;
            Self::ws_send(loser, GameServerMsg::GameEnd(false)).await?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub async fn run(&mut self) -> anyhow::Result<()> {
        // Wait for both players to acknowledge their involvement
        if !matches!(self.ws_next_active().await?, GameClientMsg::Ack) {
            bail!("No white player ack");
        };
        if !matches!(self.ws_next_passive().await?, GameClientMsg::Ack) {
            bail!("No black player ack");
        };
        self.ws_send_active(GameServerMsg::NewTurn(true)).await?;
        self.ws_send_passive(GameServerMsg::NewTurn(false)).await?;
        loop {
            let player_msg = loop {
                let player_msg = match self.ws_next_active().await {
                    Ok(msg) => msg,
                    Err(err) => {
                        self.ws_send_active(GameServerMsg::Error(format!("{:?}", err)))
                            .await?;
                        continue;
                    }
                };
                break player_msg;
            };
            match player_msg {
                GameClientMsg::TurnEnd(piece_move) => {
                    if let Err(error) = self.handle_turn_end(piece_move).await {
                        println!("Error: {:?}", error);
                        self.ws_send_active(GameServerMsg::Error(format!("{:?}", error)))
                            .await?;
                        continue;
                    };
                }
                GameClientMsg::Ack => {
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
                    self.ws_send_active(GameServerMsg::Error(format!("{:?}", error)))
                        .await?;
                }
            };
        }
        Ok(())
    }
}