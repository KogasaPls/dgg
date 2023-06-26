use crate::gui::{View, ViewMut};
use eframe::egui;
use eframe::egui::{Response, Ui, Widget};
use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize)]
pub struct ChatInputView {
    pub text: String,
}

impl ViewMut for ChatInputView {
    fn show(&mut self, ui: &mut Ui) -> Response {
        let response = ui.text_edit_multiline(&mut self.text);

        response
    }
}
