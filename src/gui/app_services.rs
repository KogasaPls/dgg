use anyhow::{bail, Context, Error, Result};
use dgg::config::ChatAppConfig;
use dgg::dgg::chat::chat_client::ChatClient;
use dgg::dgg::models::event::Event;
use dgg::dgg::utilities::cdn::CdnClient;
use std::collections::HashMap;
use std::path::PathBuf;
use std::pin::pin;
use std::sync::Arc;

use dgg::dgg::chat::chat_client;
use dgg::dgg::models::flair::Flair;
use tokio::join;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::{oneshot, Mutex};

/// A command sent to the ChatAppServices.
#[derive(Debug)]
pub enum Command {
    SendMessage(String),
}

#[derive(Debug)]
/// Receives commands from the UI, and sends events and other data back.
pub struct ChatAppServices {
    config: ChatAppConfig,
    event_tx: Sender<Event>,
    command_rx: Receiver<Command>,
    flairs_tx: oneshot::Sender<HashMap<String, Flair>>,
}

impl ChatAppServices {
    pub fn new(
        config: ChatAppConfig,
        event_tx: Sender<Event>,
        command_rx: Receiver<Command>,
        flairs_tx: oneshot::Sender<HashMap<String, Flair>>,
    ) -> Self {
        Self {
            config,
            event_tx,
            command_rx,
            flairs_tx,
        }
    }

    pub async fn start(self) {
        info!("Starting app services...");
        let mut cdn_client =
            CdnClient::new(self.config.get_cdn_url(), self.config.cache_path.clone());
        let mut chat_client = ChatClient::new(self.config);
        chat_client.connect().await.unwrap();

        let Self {
            mut event_tx,
            mut command_rx,
            mut flairs_tx,
            ..
        } = self;

        let (first, second) = join! {
            send_flairs(flairs_tx, &mut cdn_client),
            async move {
                loop {
                    handle_next_command(&mut command_rx, &mut chat_client).await;
                    emit_next_event(&mut event_tx, &mut chat_client).await;
                }
            }
        };
    }
}

async fn emit_next_event(tx: &mut Sender<Event>, chat_client: &mut ChatClient) {
    if let Some(event) = chat_client.get_next_event().await.unwrap() {
        trace!("Sending event: {:?}", event);
        tx.send(event).await.unwrap();
    }
}

async fn handle_next_command(rx: &mut Receiver<Command>, chat_client: &mut ChatClient) {
    if let Ok(command) = rx.try_recv() {
        debug!("Received command: {:?}", command);
        match command {
            Command::SendMessage(message) => {
                chat_client.send_message(message).await.unwrap();
            }
        }
    }
}

async fn send_flairs(tx: oneshot::Sender<HashMap<String, Flair>>, cdn_client: &mut CdnClient) {
    let flairs = cdn_client.get_flairs().await.unwrap();
    tx.send(flairs).unwrap();
}
