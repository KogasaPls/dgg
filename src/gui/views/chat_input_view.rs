use crate::gui::app_services::Command;
use crate::gui::{View, ViewMut};
use anyhow::Context;
use eframe::egui;
use eframe::egui::{Response, Ui, Widget};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::Sender;

#[derive(Default)]
pub struct ChatInputView {
    pub text: String,
    command_tx: Option<Sender<Command>>,
}

impl ChatInputView {
    pub fn new(command_tx: Sender<Command>) -> Self {
        Self {
            command_tx: Some(command_tx),
            ..Default::default()
        }
    }
}

impl ViewMut for ChatInputView {
    fn show(&mut self, ui: &mut Ui) -> Response {
        let response = ui.text_edit_multiline(&mut self.text);

        let sent = ui.ctx().input(|s| {
            s.events.iter().any(|e| {
                matches!(
                    e,
                    egui::Event::Key {
                        key: egui::Key::Enter,
                        pressed: true,
                        ..
                    }
                )
            })
        });

        if sent {
            if let Some(command_tx) = self.command_tx.as_ref() {
                let text = self.text.trim_end().to_string();
                command_tx
                    .blocking_send(Command::SendMessage(text))
                    .expect("Failed to send message");
            }
            self.text.clear();
        }

        response
    }
}
