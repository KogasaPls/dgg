use eframe::egui::{Response, Ui};

pub mod app;
pub mod app_services;
pub mod chat_view;

pub trait View {
    fn show(&self, ui: &mut Ui) -> Response;
}
