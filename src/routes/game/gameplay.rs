use anyhow::bail;
use axum::extract::ws::Message;
use futures::{SinkExt, StreamExt};

use super::OpenGame;

pub async fn gameplay_loop(game: OpenGame) -> anyhow::Result<()> {
    let mut is_firsts_turn = true;
    loop {
        let (active_player, passive_player) = match is_firsts_turn {
            true => (&game.user_stream.0, &game.user_stream.1),
            false => (&game.user_stream.1, &game.user_stream.0),
        };
        active_player
            .0
            .lock()
            .await
            .send(Message::Text(String::from("It's your turn")))
            .await?;
        passive_player
            .0
            .lock()
            .await
            .send(Message::Text(String::from(
                "It's not your turn, wait for your opponent's action",
            )))
            .await?;

        let Some(Ok(message)) = active_player.1.lock().await.next().await else {
            bail!("Bad message error");
        };
        is_firsts_turn = !is_firsts_turn;
    }
}
