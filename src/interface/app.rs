//! Módulo para la estructura de la aplicación en sí.

use eframe::egui::{CentralPanel, Context};
use eframe::{App, Frame};

/// La app de aerolíneas misma.
#[derive(Default)]
pub struct AerolineasApp;

impl App for AerolineasApp {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        CentralPanel::default().show(ctx, |ui| {
            ui.label("lorem ipsum");
        });
    }
}
