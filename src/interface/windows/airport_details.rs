//! Módulo de ventanas de widgets informativos.

use std::sync::Arc;

use eframe::egui::{Color32, Frame, Margin, Pos2, Ui, Window};

use crate::data::airports::airp::Airport;

/// Muestra detalles sobre un aeropuerto.
pub struct AirportDetailsWindow {
    /// El aeropuerto interno.
    airport: Arc<Option<Airport>>,

    /// Si se puede mostrar el widget o no.
    pub can_show: bool,

    /// La posición actual del widget en la pantalla.
    pub pos: Pos2,
}

impl AirportDetailsWindow {
    /// Crea una nueva instancia de los detalles del aeropuerto.
    pub fn new(airport: Option<Airport>, can_show: bool, pos: Pos2) -> Self {
        Self {
            airport: Arc::new(airport),
            can_show,
            pos,
        }
    }

    /// Setea un nuevo aeropuerto para el widget.
    pub fn set_airport(&mut self, new_airport: Arc<Option<Airport>>) {
        self.airport = new_airport;
    }

    /// Muestra por pantalla la ventana de detalles.
    pub fn show(&mut self, ui: &Ui) {
        if !self.can_show {
            return;
        }

        if let Some(airport) = self.airport.as_ref() {
            let win_frame = Frame::default()
                .fill(Color32::from_rgb(100, 100, 100))
                .outer_margin(Margin::same(3.0));
            Window::new(format!("Detalles de Aeropuerto {}", airport.name))
                .collapsible(true)
                .resizable(false)
                .title_bar(true)
                .movable(true)
                .fixed_size([200., 100.])
                .current_pos(self.pos)
                .open(&mut self.can_show)
                .fade_out(true)
                .frame(win_frame)
                .show(ui.ctx(), |ui| {
                    ui.add_space(5.0);
                });
        }
    }
}

impl Default for AirportDetailsWindow {
    fn default() -> Self {
        Self::new(None, true, Pos2::from((0., 0.)))
    }
}
