use std::{collections::VecDeque, sync::Arc};

use axum::extract::FromRef;
use tokio::{sync::Mutex, task::JoinHandle};

use crate::{routes::game::ws::GameWs, ServerState};

#[derive(Debug)]
pub struct MatchmakingPlayer {
    pub id: i32,
    pub ws: GameWs,
    pub echo: JoinHandle<()>,
}

impl MatchmakingPlayer {
    pub fn new(id: i32, ws: GameWs, echo: JoinHandle<()>) -> Self {
        MatchmakingPlayer { id, ws, echo }
    }
}

#[derive(Default, Clone, Debug)]
pub struct UserQueue {
    pub state: Arc<Mutex<VecDeque<MatchmakingPlayer>>>,
}

impl UserQueue {
    pub async fn push(&self, matchmaking_player: MatchmakingPlayer) {
        self.state.lock().await.push_back(matchmaking_player);
    }
    pub async fn pop(&self) -> Option<MatchmakingPlayer> {
        self.state.lock().await.pop_front()
    }
}

impl FromRef<ServerState> for UserQueue {
    fn from_ref(input: &ServerState) -> Self {
        input.user_queue.clone()
    }
}
