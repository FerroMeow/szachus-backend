use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};

use crate::routes::game::{matchmaking::db::Game, piece_color::PieceColor};

use super::{piece::PieceType, position::Position};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GameTurn {
    id: i32,
    turn_nr: i32,
    game: Option<i32>,
    player_color: i16,
    tile_from: String,
    tile_to: String,
    pawn_moved: String,
}

impl GameTurn {
    pub async fn create(
        db: &Pool<Postgres>,
        game: &Game,
        player: PieceColor,
        from: Position,
        to: Position,
        piece_moved: PieceType,
    ) -> anyhow::Result<GameTurn> {
        let player_color = match player {
            PieceColor::Black => 1,
            PieceColor::White => 0,
        };
        let tile_from = from.to_string();
        let tile_to = to.to_string();
        let piece_moved = piece_moved.get_name().to_owned();
        let game_id = game.id;
        let a = sqlx::query_as!(
            GameTurn,
            "INSERT INTO game_turn (game, player_color, tile_from, tile_to, pawn_moved) VALUES ($1, $2, $3, $4, $5) RETURNING *",
            game_id,
            player_color,
            tile_from,
            tile_to,
            piece_moved
        );
        a.fetch_one(db).await.map_err(|error| anyhow!(error))
    }
}
