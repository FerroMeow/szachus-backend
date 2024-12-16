use chrono::prelude::*;
use serde::{Deserialize, Serialize};

pub mod matchmaking_state;
pub mod ws_handler;

#[derive(Serialize, Deserialize, Clone)]
pub struct Game {
    id: i32,
    started_at: NaiveDateTime,
    ended_at: Option<NaiveDateTime>,
    player_black: i32,
    player_white: i32,
}
