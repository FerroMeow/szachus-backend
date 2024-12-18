use anyhow::anyhow;
use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgQueryResult, Pool, Postgres};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Game {
    pub id: i32,
    started_at: NaiveDateTime,
    ended_at: Option<NaiveDateTime>,
    player_black: i32,
    player_white: i32,
}

pub async fn create_game(
    db_pool: &Pool<Postgres>,
    username_black: i32,
    username_white: i32,
) -> anyhow::Result<Game> {
    sqlx::query_as!(
        Game,
        "INSERT INTO game (started_at, player_black, player_white) VALUES ($1, $2, $3) RETURNING *",
        Utc::now().naive_utc(),
        username_black,
        username_white
    )
    .fetch_one(db_pool)
    .await
    .map_err(|err| anyhow!(err))
}

pub async fn set_game_finished(db_pool: &Pool<Postgres>, game: Game) -> anyhow::Result<Game> {
    sqlx::query_as!(
        Game,
        "UPDATE game SET ended_at = $1 WHERE id = $2 RETURNING *",
        Utc::now().naive_utc(),
        game.id
    )
    .fetch_one(db_pool)
    .await
    .map_err(|err| anyhow!(err))
}

pub async fn remove_game(db_pool: &Pool<Postgres>, game: Game) -> anyhow::Result<PgQueryResult> {
    sqlx::query!("DELETE FROM game WHERE id = $1", game.id)
        .execute(db_pool)
        .await
        .map_err(|err| anyhow!(err))
}
