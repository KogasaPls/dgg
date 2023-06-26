use dgg::config::ChatAppConfig;

use crate::gui::app_services::ChatAppServiceData;
use crate::gui::chat_view::ChatView;
use crate::gui::View;
use anyhow::Result;
use dgg::dgg::models::event;
use dgg::dgg::models::event::Event;
use eframe::egui;
use eframe::egui::ScrollArea;
use palette::white_point::E;
use serde::{Deserialize, Serialize};
use std::sync::mpsc::Receiver;

#[derive(Default, Deserialize, Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct ChatApp {
    config: ChatAppConfig,
    #[serde(skip)]
    rx: Option<Receiver<ChatAppServiceData>>,
    chat_view: ChatView,
}

impl ChatApp {
    pub fn new(cc: &eframe::CreationContext<'_>, rx: Receiver<ChatAppServiceData>) -> Self {
        let mut app: ChatApp = match cc.storage {
            Some(storage) => eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default(),
            None => Default::default(),
        };

        app.rx = Some(rx);
        app
    }
}

impl eframe::App for ChatApp {
    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Some(rx) = &self.rx {
            for data in rx.try_iter() {
                match data {
                    ChatAppServiceData::Event(Event::ChatMessage(msg)) => {
                        self.chat_view.add_message(msg)
                    }
                    ChatAppServiceData::Flairs(flairs) => self.chat_view.set_flairs(flairs),
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
            ScrollArea::new([false, true]).show(ui, |ui| {
                self.chat_view.show(ui);
            });
        });
    }

    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
}
