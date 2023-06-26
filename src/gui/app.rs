use dgg::config::ChatAppConfig;

use crate::gui::app_services::{ChatAppServiceMessage, Command};
use crate::gui::views::chat_view;
use crate::gui::views::chat_view::ChatView;
use crate::gui::{View, ViewMut};
use anyhow::Result;
use dgg::dgg::models::event;
use dgg::dgg::models::event::Event;
use eframe::egui;
use eframe::egui::{ScrollArea, Widget};
use palette::white_point::E;
use serde::{Deserialize, Serialize};
use std::sync::mpsc::{Receiver, Sender};
use std::time::Duration;

/// The main application.
#[derive(Default)]
pub struct ChatApp {
    config: ChatAppConfig,
    rx: Option<Receiver<ChatAppServiceMessage>>,
    command_tx: Option<Sender<Command>>,
    chat_view: ChatView,
}

impl ChatApp {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        service_rx: Receiver<ChatAppServiceMessage>,
        command_tx: Sender<Command>,
    ) -> Self {
        ChatApp {
            chat_view: ChatView::new(command_tx),
            rx: Some(service_rx),
            ..Default::default()
        }
    }
}

impl eframe::App for ChatApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // todo: repaint only on events instead of on schedule
        ctx.request_repaint_after(Duration::from_millis(100));

        if let Some(rx) = &self.rx {
            for data in rx.try_iter() {
                match data {
                    ChatAppServiceMessage::Event(Event::ChatMessage(msg)) => {
                        self.chat_view.add_message(msg)
                    }
                    ChatAppServiceMessage::Flairs(flairs) => self.chat_view.set_flairs(flairs),
                    _ => Ok(()),
                }
                .expect("Failed to handle event");
            }
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
