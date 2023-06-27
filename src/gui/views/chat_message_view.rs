use crate::gui::views::chat_view::ChatView;
use crate::gui::View;
use dgg::dgg::models::emote::Emote;
use eframe::egui;
use eframe::egui::{Response, Rgba, Ui, Widget};
use egui_extras::RetainedImage;
use linkify::{Link, LinkFinder, LinkKind};
use palette::{FromColor, Hsv, Srgb};
use regex::{Captures, Regex};
use serde::{Deserialize, Serialize};
use serde_json::to_string;
use std::cell::LazyCell;
use std::collections::HashMap;
use std::fmt::Debug;
use std::process::id;
use std::rc::Rc;
use std::sync::LazyLock;

// Regex for embed links
static EMBED_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(&format!(r"#(youtube|twitch|kick)/([a-zA-Z0-9]+)")).unwrap());

static LINK_FINDER: LazyLock<LinkFinder> = LazyLock::new(|| {
    let mut link_finder = LinkFinder::new();
    link_finder.kinds(&[linkify::LinkKind::Url]);
    link_finder
});

/// A single chat message.
#[derive(Clone)]
pub struct ChatMessageView {
    pub username: String,
    pub username_color: Option<Rgba>,
    pub is_rainbow_color: bool,
    pub message: String,
    pub timestamp: String,
    pub flair_images: Vec<Rc<RetainedImage>>,
    message_with_emotes: Vec<TextOrEmoteOrLink>,
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
        let message_with_emotes = Self::parse_message(&message, emote_images);

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

    fn parse_message(
        message: &str,
        emotes: &HashMap<String, Rc<RetainedImage>>,
    ) -> Vec<TextOrEmoteOrLink> {
        let mut last_index = 0;

        let links: Vec<_> = LINK_FINDER.links(message).collect();

        let mut add_word_or_emote = |tokens: &mut Vec<TextOrEmoteOrLink>, word: &str| {
            if let Some(captures) = EMBED_REGEX.captures(word) {
                let platform = captures.get(1).unwrap().as_str().to_ascii_lowercase();
                let id = captures.get(2).unwrap().as_str().to_ascii_lowercase();
                tokens.push(TextOrEmoteOrLink::EmbedLink(EmbedLink::new(
                    word,
                    platform.as_str(),
                    id.as_str(),
                )));
            } else if let Some(emote) = emotes.get(word) {
                tokens.push(TextOrEmoteOrLink::Emote(emote.clone()));
            } else {
                tokens.push(TextOrEmoteOrLink::Text(word.to_string()));
            }
        };

        let mut tokens = Vec::new();
        for link in links {
            // Process the text before the link
            if link.start() > last_index {
                let substr = &message[last_index..link.start()];
                for word in substr.split_whitespace() {
                    add_word_or_emote(&mut tokens, word);
                }
            }

            // Process the link
            tokens.push(TextOrEmoteOrLink::Link(link.as_str().to_string()));
            last_index = link.end();
        }

        // Process the remaining text after the last link
        if last_index < message.len() {
            let substr = &message[last_index..];
            for word in substr.split_whitespace() {
                add_word_or_emote(&mut tokens, word);
            }
        }

        tokens
    }

    fn parse_message_with_emotes_no_links(
        message: &str,
        emotes: &HashMap<String, Rc<RetainedImage>>,
    ) -> Vec<TextOrEmoteOrLink> {
        let mut message_with_emotes = Vec::new();
        let mut push_text_or_emote = |current_word: String| {
            if current_word.is_empty() {
                return;
            }

            if let Some(emote) = emotes.get(&current_word) {
                message_with_emotes.push(TextOrEmoteOrLink::Emote(emote.clone()));
            } else {
                message_with_emotes.push(TextOrEmoteOrLink::Text(current_word.clone()));
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
enum TextOrEmoteOrLink {
    Text(String),
    Emote(Rc<RetainedImage>),
    Link(String),
    EmbedLink(EmbedLink),
}

#[derive(Debug, Clone)]
pub enum EmbedLinkKind {
    YouTube,
    Twitch,
    Kick,
}

#[derive(Debug, Clone)]
pub struct EmbedLink {
    pub platform: EmbedLinkKind,
    pub id: String,
    pub original: String,
}

impl EmbedLink {
    pub fn new(original: &str, platform: &str, id: &str) -> Self {
        let platform = match platform {
            "youtube" => EmbedLinkKind::YouTube,
            "twitch" => EmbedLinkKind::Twitch,
            "kick" => EmbedLinkKind::Kick,
            _ => panic!("Unknown embed platform: {}", platform),
        };

        Self {
            platform,
            id: id.to_string(),
            original: original.to_string(),
        }
    }

    pub fn url(&self) -> String {
        match self.platform {
            EmbedLinkKind::YouTube => format!("https://www.youtube.com/watch?v={}", self.id),
            EmbedLinkKind::Twitch => format!("https://www.twitch.tv/{}", self.id),
            EmbedLinkKind::Kick => format!("https://kick.tv/{}", self.id),
        }
    }

    pub fn value(&self) -> &str {
        self.original.as_str()
    }
}

impl View for TextOrEmoteOrLink {
    fn show(&self, ui: &mut Ui) -> Response {
        match self {
            TextOrEmoteOrLink::Text(text) => ui.label(text),
            TextOrEmoteOrLink::Emote(image) => {
                ui.image(image.texture_id(ui.ctx()), egui::Vec2::new(16.0, 16.0))
            }
            TextOrEmoteOrLink::Link(link) => ui.hyperlink_to(link, link),
            TextOrEmoteOrLink::EmbedLink(link) => ui.hyperlink_to(link.value(), link.url()),
        }
    }
}
