use dgg::config::ChatAppConfig;
use std::collections::HashMap;

use crate::gui::app_services::Command;
use crate::gui::views::chat_view;
use crate::gui::views::chat_view::ChatView;
use crate::gui::{View, ViewMut};
use anyhow::{bail, Result};
use dgg::dgg::models::event;
use dgg::dgg::models::event::Event;
use dgg::dgg::models::flair::Flair;
use eframe::egui;
use eframe::egui::{ScrollArea, Widget};
use futures_util::TryFutureExt;
use palette::white_point::E;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::sync::mpsc::error::TryRecvError;
use tokio::sync::{mpsc, oneshot};

/// The main application.
#[derive(Default)]
pub struct ChatApp {
    config: ChatAppConfig,
    event_rx: Option<mpsc::Receiver<Event>>,
    flairs_rx: Option<oneshot::Receiver<HashMap<String, Flair>>>,
    chat_view: ChatView,
}

impl ChatApp {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        event_rx: mpsc::Receiver<Event>,
        command_tx: mpsc::Sender<Command>,
        flairs_rx: oneshot::Receiver<HashMap<String, Flair>>,
    ) -> Self {
        ChatApp {
            chat_view: ChatView::new(command_tx),
            event_rx: Some(event_rx),
            flairs_rx: Some(flairs_rx),
            ..Default::default()
        }
    }
}

impl eframe::App for ChatApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // todo: repaint only on events instead of on schedule
        ctx.request_repaint_after(Duration::from_millis(100));

        if let Some(flairs_rx) = &mut self.flairs_rx {
            if let Ok(flairs) = flairs_rx.try_recv() {
                self.chat_view
                    .set_flairs(flairs)
                    .expect("Failed to set flairs");
                self.flairs_rx = None;
            }
        }

        if let Some(event_rx) = self.event_rx.as_mut() {
            handle_event(event_rx, &mut self.chat_view).unwrap_or_else(|e| {
                panic!("Error handling event: {:?}", e);
            });
        }

        #[cfg(not(target_arch = "wasm32"))] // no File->Quit on web pages!
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        _frame.close();
                    }
                });
                ui.heading("Destiny.gg Chat");
                egui::warn_if_debug_build(ui);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            self.chat_view.show(ui);
        });
    }
}

fn handle_event(event_rx: &mut mpsc::Receiver<Event>, chat_view: &mut ChatView) -> Result<()> {
    match event_rx.try_recv() {
        Ok(data) => match data {
            Event::ChatMessage(msg) => chat_view.add_message(msg),
            _ => Ok(()),
        },
        Err(TryRecvError::Empty) => Ok(()),
        Err(e) => {
            panic!("Error receiving event: {:?}", e);
        }
    };

    Ok(())
}
