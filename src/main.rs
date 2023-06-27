#![feature(lazy_cell)]
#![warn(clippy::all, rust_2018_idioms)]
#![allow(unused)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

#[macro_use]
extern crate log;

pub mod gui;

use crate::gui::app::ChatApp;
use crate::gui::app_services::ChatAppServices;
use futures_util::task::SpawnExt;
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

fn main() -> eframe::Result<()> {
    init();

    let (event_tx, event_rx) = mpsc::channel(100);
    let (command_tx, command_rx) = mpsc::channel(100);
    let (flairs_tx, flairs_rx) = oneshot::channel();
    let (emotes_tx, emotes_rx) = oneshot::channel();

    let config = ChatAppConfig::load();
    let services = ChatAppServices::new(config, event_tx, command_rx, flairs_tx, emotes_tx);

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
        Box::new(|cc| Box::new(ChatApp::new(cc, event_rx, command_tx, flairs_rx, emotes_rx))),
    )
}
