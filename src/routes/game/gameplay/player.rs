use crate::routes::game::ws::GameWs;

#[derive(Debug)]
pub struct GamePlayer {
    pub id: i32,
    pub ws: GameWs,
}

impl GamePlayer {
    pub fn new(id: i32, ws: GameWs) -> Self {
        Self { id, ws }
    }
}
