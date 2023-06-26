use eframe::egui::{Response, Ui};

pub mod app;
pub mod app_services;
pub mod views;

/// Like `eframe::egui::Widget` but doesn't take ownership of self.
pub trait View {
    fn show(&self, ui: &mut Ui) -> Response;
}

pub trait ViewMut {
    fn show(&mut self, ui: &mut Ui) -> Response;
}
