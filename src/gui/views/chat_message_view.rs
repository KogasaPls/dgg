use crate::gui::views::chat_view::ChatView;
use crate::gui::View;
use eframe::egui;
use eframe::egui::{Response, Rgba, Ui, Widget};
use egui_extras::RetainedImage;
use palette::{FromColor, Hsv, Srgb};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::rc::Rc;

/// A single chat message.
#[derive(Clone)]
pub struct ChatMessageView {
    pub username: String,
    pub username_color: Option<Rgba>,
    pub is_rainbow_color: bool,
    pub message: String,
    pub timestamp: String,
    pub flair_images: Vec<Rc<RetainedImage>>,
}

impl Debug for ChatMessageView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChatMessageView")
            .field("username", &self.username)
            .field("username_color", &self.username_color)
            .field("is_rainbow_color", &self.is_rainbow_color)
            .field("message", &self.message)
            .field("timestamp", &self.timestamp)
            .finish()
    }
}

impl ChatMessageView {
    fn show_flairs(&self, ui: &mut Ui) {
        for image in &self.flair_images {
            ui.image(image.texture_id(ui.ctx()), egui::Vec2::new(16.0, 16.0));
        }
    }

    fn show_username(&self, ui: &mut Ui) {
        if self.is_rainbow_color {
            let len = self.username.len() as f32;

            for (i, c) in self.username.chars().enumerate() {
                let hue = i as f32 / len;
                let color: Srgb = Srgb::from_color(Hsv::new(hue * 360.0, 1.0, 1.0));

                let color = egui::Color32::from_rgb(
                    (color.red * 255.0) as u8,
                    (color.green * 255.0) as u8,
                    (color.blue * 255.0) as u8,
                );

                ui.colored_label(color, &c.to_string());
            }
        } else if let Some(color) = self.username_color {
            ui.colored_label(color, &self.username);
        } else {
            ui.label(&self.username);
        }
    }
}

impl View for ChatMessageView {
    fn show(&self, ui: &mut Ui) -> Response {
        ui.horizontal_wrapped(|ui| {
            ui.label(&self.timestamp);
            ui.separator();
            self.show_flairs(ui);
            self.show_username(ui);
            ui.separator();
            ui.label(&self.message)
        })
        .response
    }
}
