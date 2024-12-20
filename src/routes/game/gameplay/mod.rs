use anyhow::bail;
use axum::extract::ws::Message;
use chessboard::ChessBoard;
use db::{increase_winner_score, set_game_finished, GameTurn};
use player::GamePlayer;
use sqlx::{Pool, Postgres};
use ws_message::GameServerMsg;

use super::matchmaking::db::Game;
use super::opponent_pair::OpponentPair;

use super::piece_color::PieceColor;
use super::ws::GameWs;
use super::ws_messages::{ChessMove, GameClientMsg, ServerMsg};

pub mod chessboard;
pub mod db;
pub mod piece;
pub mod player;
pub mod position;
pub mod ws_message;

#[derive(Debug)]
pub struct Gameplay {
    db_pool: Pool<Postgres>,
    pub game_data: Game,
    chess_board: ChessBoard,
    pub players: OpponentPair,
    turn_number: i32,
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

    pub fn new(db_pool: Pool<Postgres>, game_data: Game, players: OpponentPair) -> Self {
        Self {
            db_pool,
            game_data,
            chess_board: ChessBoard::new(),
            players,
            turn_number: 1,
        }
    }

    async fn ws_send_active(&mut self, msg: GameServerMsg) -> anyhow::Result<()> {
        Self::ws_send(&self.players.get_active().ws, msg).await
    }

    async fn ws_send_passive(&mut self, msg: GameServerMsg) -> anyhow::Result<()> {
        Self::ws_send(&self.players.get_passive().ws, msg).await
    }

    async fn ws_next_active(&mut self) -> anyhow::Result<GameClientMsg> {
        Self::ws_next(&self.players.get_active().ws).await
    }

    async fn ws_next_passive(&mut self) -> anyhow::Result<GameClientMsg> {
        Self::ws_next(&self.players.get_passive().ws).await
    }

    async fn handle_turn_end(&mut self, piece_move: ChessMove) -> anyhow::Result<()> {
        let player_color = self.players.current_player_color;
        let piece_move = piece_move.maybe_invert(player_color);
        let (piece_type, removed_piece_maybe) = self
            .chess_board
            .move_piece(
                player_color,
                piece_move.position_from,
                piece_move.position_to,
            )
            .await?;
        GameTurn::create(
            &self.db_pool,
            &self.game_data,
            self.turn_number,
            player_color,
            piece_move.position_from,
            piece_move.position_to,
            piece_type,
        )
        .await?;
        let removed_piece_to =
            removed_piece_maybe.map(|piece| (piece.color, piece_move.position_to));
        self.players
            .white_player
            .ws
            .send_as_text(&ServerMsg::Game(GameServerMsg::PawnMove(
                piece_move,
                removed_piece_to,
            )))
            .await?;
        self.players
            .black_player
            .ws
            .send_as_text(&ServerMsg::Game(GameServerMsg::PawnMove(
                piece_move.invert(),
                removed_piece_to.map(|to| (to.0, to.1.invert())),
            )))
            .await?;
        Ok(())
    }

    async fn switch_turns(&mut self) -> anyhow::Result<()> {
        self.players.switch_active();
        let _ = self.ws_send_active(GameServerMsg::NewTurn(true)).await;
        let _ = self.ws_send_passive(GameServerMsg::NewTurn(false)).await;
        self.turn_number += 1;
        Ok(())
    }

    async fn handle_win(&self) -> anyhow::Result<Option<&GamePlayer>> {
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
            Self::ws_send(&winner.ws, GameServerMsg::GameEnd(true)).await?;
            Self::ws_send(&loser.ws, GameServerMsg::GameEnd(false)).await?;
            Ok(Some(winner))
        } else {
            Ok(None)
        }
    }

    pub async fn run(&mut self) -> anyhow::Result<()> {
        // Wait for both players to acknowledge their involvement
        let Ok(GameClientMsg::Ack) = self.ws_next_active().await else {
            bail!("No white player ack");
        };
        if !matches!(self.ws_next_passive().await?, GameClientMsg::Ack) {
            bail!("No black player ack");
        };
        self.ws_send_active(GameServerMsg::NewTurn(true)).await?;
        self.ws_send_passive(GameServerMsg::NewTurn(false)).await?;
        let winner = loop {
            match self.ws_next_active().await? {
                GameClientMsg::TurnEnd(piece_move) => {
                    if let Err(error) = self.handle_turn_end(piece_move).await {
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
                Ok(None) => {
                    self.switch_turns().await?;
                }
                Ok(Some(winner)) => {
                    break winner;
                }
                Err(error) => {
                    self.ws_send_active(GameServerMsg::Error(format!("{:?}", error)))
                        .await?;
                }
            };
        };
        set_game_finished(&self.db_pool, &self.game_data)
            .await
            .unwrap();
        increase_winner_score(&self.db_pool, winner).await?;
        Ok(())
    }
}
