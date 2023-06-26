use anyhow::{anyhow, bail, Context, Result};

use dgg::dgg::models::event::{ChatMessageData, EventData};
use dgg::dgg::models::flair::{Flair, FlairImage};

use crate::gui::View;
use cached::CachedAsync;
use eframe::egui;
use eframe::egui::{ImageData, Response, Rgba, TextBuffer, TextureHandle, Ui};
use egui_extras::image::load_image_bytes;
use egui_extras::RetainedImage;
use palette::convert::TryIntoColor;
use palette::rgb::Rgb;
use palette::{FromColor, Hsv, IntoColor, Srgb, Srgba};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::default;
use std::fmt::Debug;
use std::ops::Deref;
use std::rc::Rc;
use std::sync::Arc;
use url::quirks::username;

#[derive(Clone, Serialize, Deserialize)]
pub struct ChatMessageView {
    pub username: String,
    pub username_color: Option<Rgba>,
    pub is_rainbow_color: bool,
    pub message: String,
    pub timestamp: String,

    #[serde(skip)]
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
        if let Some(color) = self.username_color {
            ui.colored_label(color, &self.username);
        } else if self.is_rainbow_color {
            let len = self.username.len() as f32;

            for (i, c) in self.username.chars().enumerate() {
                let hue = i as f32 / len;
                let color: Srgb = Srgb::from_color(Hsv::new(hue * 360.0, 1.0, 1.0));

                let color = egui::Color32::from_rgb(
                    (color.red) as u8,
                    (color.green) as u8,
                    (color.blue) as u8,
                );

                ui.colored_label(color, &c.to_string());
            }
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

#[derive(Default, Serialize, Deserialize)]
pub struct ChatView {
    #[serde(skip)]
    messages: Vec<ChatMessageView>,
    #[serde(skip)]
    user_styles: HashMap<String, Option<UserStyle>>,
    default_username_color: Rgba,

    #[serde(skip)]
    flairs: HashMap<String, Rc<Flair>>,

    #[serde(skip)]
    flair_images: HashMap<String, Rc<RetainedImage>>,
}

impl ChatView {
    pub fn set_flairs(&mut self, flairs: HashMap<String, Flair>) -> Result<()> {
        debug!("Updating {} flairs", flairs.len());

        self.flairs = flairs.into_iter().map(|(k, v)| (k, Rc::new(v))).collect();
        self.flair_images.clear();

        for flair in self.flairs.values() {
            if flair.image.is_empty() {
                continue;
            }

            let maybe_bytes = flair.image[0].bytes.as_ref();
            if maybe_bytes.is_none() {
                continue;
            }

            let key = Self::get_flair_image_key(flair)?;
            let bytes = maybe_bytes.ok_or(anyhow!("Flair image has no bytes"))?;
            let image = RetainedImage::from_color_image(
                key.clone(),
                load_image_bytes(bytes)
                    .ok()
                    .ok_or(anyhow!("Failed to load image"))?,
            );

            self.flair_images.insert(key, Rc::new(image));
        }
        Ok(())
    }

    pub fn show_flair_image(&mut self, ui: &mut Ui, flair: &Flair) -> Result<()> {
        let key = Self::get_flair_image_key(flair)?;

        let texture = self
            .flair_images
            .get(key.as_str())
            .context("Flair image is not loaded")
            .unwrap();

        texture.show(ui);

        Ok(())
    }

    fn get_flair_image_key(flair: &Flair) -> Result<String> {
        if flair.image.is_empty() {
            bail!("Flair {} has no image", flair.name);
        }

        Ok(Self::get_flair_image_key_str(flair.name.as_str()))
    }

    fn get_flair_image_key_str(flair_name: &str) -> String {
        format!("flair_{}", flair_name)
    }
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
struct UserStyle {
    pub is_rainbow: bool,
    pub color: Option<Rgba>,
    pub flairs: Vec<Rc<Flair>>,
}

impl UserStyle {
    fn new(mut flairs: Vec<Rc<Flair>>) -> Self {
        let mut user_style = Self::default();

        if !flairs.is_empty() {
            flairs.sort_by(|first, second| first.priority.cmp(&second.priority));

            for flair in flairs.iter().filter(|f| !f.hidden).cloned() {
                if flair.rainbow_color {
                    user_style.is_rainbow = true;
                } else if !user_style.is_rainbow
                    && user_style.color.is_none()
                    && flair.color.is_some()
                {
                    user_style.color = Some(Rgba::from_rgba_premultiplied(
                        flair.color.unwrap().red,
                        flair.color.unwrap().green,
                        flair.color.unwrap().blue,
                        1.0,
                    ));
                }

                user_style.flairs.push(flair);
            }
        }

        if user_style.color.is_none() {
            user_style.color = Some(Rgba::from_rgb(1.0, 1.0, 1.0));
        }

        user_style
    }
}

impl ChatView {
    pub fn new() -> Self {
        Self {
            default_username_color: Rgba::from_rgb(1.0, 1.0, 1.0),
            ..Default::default()
        }
    }

    pub fn add_message(&mut self, msg: EventData<ChatMessageData>) -> Result<()> {
        let user = msg.base.user.context("Message has no user")?;
        let user_style = self
            .get_user_style(user.nick.clone(), user.features)?
            .unwrap_or_default();

        let timestamp = msg
            .base
            .timestamp
            .context("Message has no timestamp")?
            .format("%H:%M")
            .to_string();

        let flair_images = user_style
            .flairs
            .iter()
            .filter(|f| !f.image.is_empty())
            .map(|f| {
                self.flair_images
                    .get(Self::get_flair_image_key(f).unwrap().as_str())
                    .cloned()
                    .expect(format!("Flair has no image data: {}.", f.name).as_str())
            })
            .collect::<Vec<Rc<RetainedImage>>>();

        let view = ChatMessageView {
            message: msg.data.data,
            username: user.nick,
            username_color: user_style.color,
            is_rainbow_color: user_style.is_rainbow,
            timestamp,
            flair_images,
        };

        debug!("Adding message {:?}", view);

        self.messages.push(view);

        Ok(())
    }

    fn get_user_style(
        &mut self,
        username: String,
        flairs: Vec<String>,
    ) -> Result<Option<UserStyle>> {
        if self.user_styles.contains_key(username.as_str()) {
            let user_style: Option<UserStyle> =
                self.user_styles.get(username.as_str()).unwrap().clone();

            return Ok(user_style);
        }

        if self.flairs.is_empty() {
            return Ok(None);
        }

        let mut flairs = flairs
            .into_iter()
            .map(|f| {
                self.flairs
                    .get(&f)
                    .context("Flair not found")
                    .map(|f| f.clone())
            })
            .filter_map(|f| f.ok())
            .collect::<Vec<Rc<Flair>>>();

        let user_style = UserStyle::new(flairs);
        self.user_styles.insert(username, Some(user_style.clone()));
        Ok(Some(user_style))
    }
}

impl View for ChatView {
    fn show(&self, ui: &mut Ui) -> Response {
        ui.vertical(|ui| {
            for message in self.messages.iter() {
                message.show(ui);
            }
        })
        .response
    }
}
