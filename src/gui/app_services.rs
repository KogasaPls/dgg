use anyhow::{bail, Context, Error, Result};
use dgg::config::ChatAppConfig;
use dgg::dgg::chat::chat_client::ChatClient;
use dgg::dgg::models::event::Event;
use dgg::dgg::utilities::cdn::CdnClient;
use std::collections::HashMap;
use std::path::PathBuf;

use dgg::dgg::models::flair::Flair;
use std::sync::mpsc::{Receiver, Sender};

/// A command sent to the ChatAppServices.
#[derive(Debug)]
pub enum Command {
    SendMessage(String),
}

/// A message sent from the ChatAppServices.
#[derive(Debug)]
pub enum ChatAppServiceMessage {
    Event(Event),
    Flairs(HashMap<String, Flair>),
    Command(Command),
}

#[derive(Debug)]
/// Responsible for emitting data (events, asynchronously loaded data, etc.) for the GUI.
pub struct ChatAppServices {
    config: ChatAppConfig,
    tx: Sender<ChatAppServiceMessage>,
    rx: Option<Receiver<Command>>,
    chat_client: Option<ChatClient>,
    cdn_client: Option<CdnClient>,
}

impl ChatAppServices {
    pub fn new(
        config: ChatAppConfig,
        service_tx: Sender<ChatAppServiceMessage>,
        command_rx: Option<Receiver<Command>>,
    ) -> Self {
        Self {
            config,
            tx: service_tx,
            rx: command_rx,
            chat_client: None,
            cdn_client: None,
        }
    }

    pub async fn start_async(mut self) -> Result<()> {
        info!("Starting app services...");
        self.initialize_async().await?;
        self.send_flairs().await?;
        self.handle_events().await;
        Ok(())
    }

    async fn send_flairs(&mut self) -> Result<()> {
        debug!("Sending flairs...");
        let cdn_client = self.get_cdn_client_mut().unwrap();
        let mut flairs = cdn_client.get_flairs().await.unwrap();
        self.send_data(ChatAppServiceMessage::Flairs(flairs)).await
    }

    async fn handle_events(mut self) {
        debug!("Handling events...");
        tokio::spawn(async move {
            loop {
                self.handle_commands();
                self.handle_next_event().await.unwrap();
            }
        });
    }

    async fn initialize_async(&mut self) -> Result<()> {
        debug!("Initializing app services");
        self.initialize_chat_client().await?;
        self.initialize_cdn_client()?;
        Ok(())
    }

    fn get_cdn_client(&self) -> Result<&CdnClient> {
        self.cdn_client
            .as_ref()
            .context("CDN client not initialized")
    }

    fn get_cdn_client_mut(&mut self) -> Result<&mut CdnClient> {
        self.cdn_client
            .as_mut()
            .context("CDN client not initialized")
    }

    fn get_chat_client(&self) -> Result<&ChatClient> {
        self.chat_client
            .as_ref()
            .context("Chat client not initialized")
    }

    fn get_chat_client_mut(&mut self) -> Result<&mut ChatClient> {
        self.chat_client
            .as_mut()
            .context("Chat client not initialized")
    }

    async fn initialize_chat_client(&mut self) -> Result<()> {
        debug!("Initializing chat client");
        let mut chat_client = ChatClient::new(self.config.clone());
        chat_client.connect().await?;
        self.chat_client = Some(chat_client);

        Ok(())
    }

    fn initialize_cdn_client(&mut self) -> Result<()> {
        debug!("Initializing CDN client");
        let cdn_url = self.config.cdn_url.clone().context("CDN URL not set")?;
        let cdn_client = CdnClient::new(cdn_url, self.config.cache_path.clone());
        self.cdn_client = Some(cdn_client);

        Ok(())
    }

    async fn handle_commands(&mut self) -> Result<()> {
        trace!("Handling commands...");

        if let Some(rx) = &mut self.rx {
            if let Ok(command) = rx.try_recv() {
                debug!("Received command: {:?}", command);
                match command {
                    Command::SendMessage(message) => {
                        self.get_chat_client_mut()?.send_message(message).await?;
                    }
                }
            }
        }

        Ok(())
    }

    async fn handle_next_event(&mut self) -> Result<()> {
        trace!("Waiting for event...");

        let event = self
            .get_chat_client_mut()?
            .get_next_event()
            .await?
            .context("No event received")?;

        self.send_data(ChatAppServiceMessage::Event(event)).await?;

        Ok(())
    }

    async fn send_data(&mut self, data: ChatAppServiceMessage) -> Result<()> {
        self.tx.send(data)?;
        Ok(())
    }
}
