use crate::gui::views::chat_view::ChatView;
use crate::gui::View;
use dgg::dgg::models::emote::Emote;
use eframe::egui;
use eframe::egui::{Response, Rgba, Ui, Widget};
use egui_extras::RetainedImage;
use palette::{FromColor, Hsv, Srgb};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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
    pub message_with_emotes: Vec<TextOrEmote>,
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
    pub fn new(
        username: String,
        username_color: Option<Rgba>,
        is_rainbow_color: bool,
        message: String,
        timestamp: String,
        flair_images: Vec<Rc<RetainedImage>>,
        emote_images: &HashMap<String, Rc<RetainedImage>>,
    ) -> Self {
        let message_with_emotes = Self::parse_message_with_emotes(&message, emote_images);

        Self {
            username,
            username_color,
            is_rainbow_color,
            message,
            timestamp,
            flair_images,
            message_with_emotes,
        }
    }

    fn parse_message_with_emotes(
        message: &str,
        emotes: &HashMap<String, Rc<RetainedImage>>,
    ) -> Vec<TextOrEmote> {
        let mut message_with_emotes = Vec::new();
        let mut push_text_or_emote = |current_word: String| {
            if current_word.is_empty() {
                return;
            }

            if let Some(emote) = emotes.get(&current_word) {
                message_with_emotes.push(TextOrEmote::Emote(emote.clone()));
            } else {
                message_with_emotes.push(TextOrEmote::Text(current_word.clone()));
            }
        };

        let mut current_word = String::new();
        for c in message.chars() {
            if c == ' ' {
                // We're at the end of a word, check if it's an emote
                push_text_or_emote(current_word.clone());
                current_word.clear();
            } else {
                current_word.push(c);
            }
        }

        // After we've processed all the characters, check if there's any remaining word
        push_text_or_emote(current_word.clone());

        message_with_emotes
    }

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

    fn show_message(&self, ui: &mut Ui) {
        for text_or_emote in &self.message_with_emotes {
            text_or_emote.show(ui);
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
            self.show_message(ui);
        })
        .response
    }
}

#[derive(Clone)]
pub enum TextOrEmote {
    Text(String),
    Emote(Rc<RetainedImage>),
}

impl View for TextOrEmote {
    fn show(&self, ui: &mut Ui) -> Response {
        match self {
            TextOrEmote::Text(text) => ui.label(text),
            TextOrEmote::Emote(image) => {
                ui.image(image.texture_id(ui.ctx()), egui::Vec2::new(16.0, 16.0))
            }
        }
    }
}
