use std::{collections::HashMap, sync::Arc};

use axum::extract::FromRef;
use tokio::{
    sync::{Mutex, MutexGuard},
    task::JoinHandle,
};

use crate::{routes::game::ws::GameWs, ServerState};

#[derive(Clone, Debug)]
pub struct MatchmakingPlayer {
    pub ws: GameWs,
    pub echo_task: Arc<JoinHandle<()>>,
}

impl MatchmakingPlayer {
    pub fn new(ws: GameWs, echo_task: Arc<JoinHandle<()>>) -> Self {
        MatchmakingPlayer { ws, echo_task }
    }
}

#[derive(Default, Clone, Debug)]
pub struct UserQueue {
    state: Arc<Mutex<HashMap<i32, MatchmakingPlayer>>>,
}

impl UserQueue {
    pub async fn get(&self) -> MutexGuard<HashMap<i32, MatchmakingPlayer>> {
        self.state.lock().await
    }
}

impl FromRef<ServerState> for UserQueue {
    fn from_ref(input: &ServerState) -> Self {
        input.user_queue.clone()
    }
}
