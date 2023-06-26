use anyhow::{bail, Context, Error, Result};
use dgg::config::ChatAppConfig;
use dgg::dgg::chat::chat_client::ChatClient;
use dgg::dgg::models::event::Event;
use dgg::dgg::utilities::cdn::CdnClient;
use std::collections::HashMap;
use std::path::PathBuf;

use dgg::dgg::models::flair::Flair;
use std::sync::mpsc::Sender;

#[derive(Debug)]
pub enum ChatAppServiceData {
    Event(Event),
    Flairs(HashMap<String, Flair>),
}

#[derive(Debug)]
// Responsible for emitting Events to and receiving Commands from the GUI, as well as
// providing access to data from the CDN.
pub struct ChatAppServices {
    config: ChatAppConfig,
    tx: Sender<ChatAppServiceData>,
    chat_client: Option<ChatClient>,
    cdn_client: Option<CdnClient>,
}

impl ChatAppServices {
    pub fn new(config: ChatAppConfig, tx: Sender<ChatAppServiceData>) -> Self {
        Self {
            config,
            tx,
            chat_client: None,
            cdn_client: None,
        }
    }

    pub async fn start_async(mut self) -> Result<()> {
        info!("Starting app services...");
        self.initialize_async().await?;
        self.handle_events().await;
        Ok(())
    }

    async fn handle_events(mut self) {
        debug!("Handling events...");
        tokio::spawn(async move {
            let cdn_client = self.get_cdn_client_mut().unwrap();
            let mut flairs = cdn_client.get_flairs().await.unwrap();
            trace!("Got flairs: {:?}", flairs);

            self.send_data(ChatAppServiceData::Flairs(flairs))
                .await
                .unwrap();

            loop {
                trace!("Waiting for event...");
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

    async fn handle_next_event(&mut self) -> Result<()> {
        let event = self
            .get_chat_client_mut()?
            .get_next_event()
            .await?
            .context("No event received")?;

        self.send_data(ChatAppServiceData::Event(event)).await?;

        Ok(())
    }

    async fn send_data(&mut self, data: ChatAppServiceData) -> Result<()> {
        debug!("Sending data: {:?}", data);
        self.tx.send(data)?;
        Ok(())
    }
}
