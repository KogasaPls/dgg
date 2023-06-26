use anyhow::{anyhow, bail, Context, Result};

use dgg::dgg::models::event::{ChatMessageData, EventData};
use dgg::dgg::models::flair::Flair;

use crate::gui::app_services::Command;
use crate::gui::views::chat_input_view::ChatInputView;
use crate::gui::views::chat_message_view::ChatMessageView;
use crate::gui::{View, ViewMut};
use eframe::egui;
use eframe::egui::panel::TopBottomSide::Bottom;
use eframe::egui::{
    Align, Response, Rgba, ScrollArea, TextBuffer, TextStyle, TopBottomPanel, Ui, Widget,
};
use egui_extras::image::load_image_bytes;
use egui_extras::RetainedImage;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Debug;
use std::rc::Rc;
use std::sync::mpsc::Sender;

/// The main chat view, consisting of a list of [ChatMessageView]s and a [ChatInputView].
#[derive(Default)]
pub struct ChatView {
    chat_input_view: ChatInputView,

    messages: Vec<ChatMessageView>,
    user_styles: HashMap<String, Option<UserStyle>>,
    default_username_color: Rgba,
    flairs: HashMap<String, Rc<Flair>>,
    flair_images: HashMap<String, Rc<RetainedImage>>,

    command_tx: Option<Sender<Command>>,
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

#[derive(Debug, Default, Clone)]
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
    pub fn new(command_tx: Sender<Command>) -> Self {
        Self {
            default_username_color: Rgba::from_rgb(1.0, 1.0, 1.0),
            command_tx: Some(command_tx.clone()),
            chat_input_view: ChatInputView::new(command_tx),
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

impl ViewMut for ChatView {
    fn show(&mut self, ui: &mut Ui) -> Response {
        ui.vertical_centered(|ui| {
            ui.vertical(|ui| {
                ScrollArea::new([false, true]).show_rows(
                    ui,
                    ui.text_style_height(&TextStyle::Body),
                    self.messages.len(),
                    |ui, rows| {
                        for row in rows.start..rows.end {
                            self.messages[row].show(ui);
                        }
                    },
                );
            });

            TopBottomPanel::new(Bottom, "chat-input-view").show(ui.ctx(), |ui| {
                ui.add_sized([ui.available_width(), 80.0], |ui: &mut Ui| {
                    self.chat_input_view.show(ui)
                });

                ui.add_space(10.0);
            });
        })
        .response
    }
}
