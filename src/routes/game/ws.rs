use std::sync::Arc;

use anyhow::anyhow;
use axum::extract::ws::{Message, WebSocket};
use futures::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use serde::Serialize;
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub struct GameWs {
    rx: Arc<Mutex<SplitStream<WebSocket>>>,
    tx: Arc<Mutex<SplitSink<WebSocket, Message>>>,
}

impl GameWs {
    pub fn new(ws: WebSocket) -> Self {
        let (tx, rx) = ws.split();
        GameWs {
            rx: Arc::new(Mutex::new(rx)),
            tx: Arc::new(Mutex::new(tx)),
        }
    }

    pub async fn get(&self) -> anyhow::Result<Message> {
        self.rx
            .lock()
            .await
            .next()
            .await
            .ok_or(anyhow!("No message"))?
            .map_err(|err| anyhow!(err))
    }

    pub async fn send(&self, message: Message) -> anyhow::Result<()> {
        self.tx
            .lock()
            .await
            .send(message)
            .await
            .map_err(|err| anyhow!(err))
    }

    pub async fn send_as_text<T: Sized + Serialize>(&self, message: &T) -> anyhow::Result<()> {
        let serialized = serde_json::to_string(message)?;
        let message = Message::Text(serialized);
        self.send(message).await
    }
}
