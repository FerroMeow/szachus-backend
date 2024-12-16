use anyhow::bail;
use axum::extract::ws::Message;

use super::piece::PieceColor;

use super::ws::GameWs;
use super::ws_messages::{ChessMove, GameClientMsg, GameServerMsg, ServerMsg};
use super::OpenGame;

pub struct Gameplay {
    game: OpenGame,
    active_player: (PieceColor, GameWs),
    passive_player: (PieceColor, GameWs),
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

    async fn ws_send_active(&mut self, msg: GameServerMsg) -> anyhow::Result<()> {
        Self::ws_send(&self.active_player.1, msg).await
    }

    async fn ws_send_passive(&mut self, msg: GameServerMsg) -> anyhow::Result<()> {
        Self::ws_send(&self.passive_player.1, msg).await
    }

    async fn ws_next_active(&mut self) -> anyhow::Result<GameClientMsg> {
        Self::ws_next(&self.active_player.1).await
    }

    async fn ws_next_passive(&mut self) -> anyhow::Result<GameClientMsg> {
        Self::ws_next(&self.passive_player.1).await
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

    async fn handle_turn_end(&mut self, piece_move: ChessMove) -> anyhow::Result<()> {
        let removed_piece_maybe = self
            .game
            .chess_board
            .move_piece(
                &self.active_player.0,
                &piece_move.position_from,
                &piece_move.position_to,
            )
            .await?;
        let removed_piece_to =
            removed_piece_maybe.map(|lock| (lock.color.clone(), piece_move.clone().position_to));
        self.ws_send_active(GameServerMsg::MovedCorrectly(removed_piece_to.clone()))
            .await?;
        self.ws_send_passive(GameServerMsg::PawnMove(piece_move, removed_piece_to))
            .await?;
        Ok(())
    }

    async fn switch_turns(&mut self) -> anyhow::Result<()> {
        std::mem::swap(&mut self.active_player, &mut self.passive_player);
        self.ws_send_active(GameServerMsg::NewTurn(true)).await?;
        self.ws_send_passive(GameServerMsg::NewTurn(false)).await?;
        Ok(())
    }

    async fn handle_win(&mut self) -> anyhow::Result<bool> {
        let white_king = self.game.chess_board.find_king(PieceColor::White);
        let black_king = self.game.chess_board.find_king(PieceColor::Black);
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
            Self::ws_send(&winner.1, GameServerMsg::GameEnd(true)).await?;
            Self::ws_send(&loser.1, GameServerMsg::GameEnd(false)).await?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}
