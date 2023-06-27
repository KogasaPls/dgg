#![feature(lazy_cell)]
#![warn(clippy::all, rust_2018_idioms)]
#![allow(unused)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

#[macro_use]
extern crate log;

pub mod gui;

use crate::gui::app::ChatApp;
use crate::gui::app_services::ChatAppServices;
use futures_util::SinkExt;
use tokio::sync::{mpsc, oneshot};

use dgg::config::ChatAppConfig;
use dgg::dgg::chat::chat_client::ChatClient;

fn init() {
    dotenv::dotenv().ok();
    pretty_env_logger::formatted_timed_builder()
        .parse_env("RUST_LOG")
        .init();

    info!("Starting...");
}

#[tokio::main]
async fn test() {
    init();

    let config = ChatAppConfig::load();
    let mut client = ChatClient::new(config);
    client.connect().await.expect("Failed to connect");

    while let Some(msg) = client.get_next_message().await.unwrap() {
        debug!("Received event: {:?}", msg);
    }

    debug!("Connected");
}

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    init();

    let config = ChatAppConfig::load();

    let (event_tx, event_rx) = mpsc::channel(100);
    let (command_tx, command_rx) = mpsc::channel(100);
    let (flairs_tx, flairs_rx) = oneshot::channel();
    let services = ChatAppServices::new(config, event_tx, command_rx, flairs_tx);

    let tokio = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .thread_name("dgg-chat-app-worker")
        .worker_threads(2)
        .build()
        .unwrap();

    tokio.spawn(async move {
        services.start().await;
    });

    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Destiny.gg Chat",
        native_options,
        Box::new(|cc| Box::new(ChatApp::new(cc, event_rx, command_tx, flairs_rx))),
    )
}

// When compiling to web using trunk:
#[cfg(target_arch = "wasm32")]
fn main() {
    // Redirect `log` message to `console.log` and friends:
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        eframe::WebRunner::new()
            .start(
                "the_canvas_id", // hardcode it
                web_options,
                Box::new(|cc| Box::new(eframe_template::ChatApp::new(cc))),
            )
            .await
            .expect("failed to start eframe");
    });
}
