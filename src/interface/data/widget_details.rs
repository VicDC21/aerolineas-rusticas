//! Módulo para persisitir las configs de algunos widgets.

use crate::interface::windows::airport_details::AirportDetailsWindow;

/// Detalles de algunos widgets que necesitan persistir entre ciclos.
pub struct WidgetDetails {
    /// Detalles sobre un aeropuerto.
    pub airp_details: AirportDetailsWindow,
}

impl WidgetDetails {
    /// Crea una nueva instancia de los detalles de widgets.
    pub fn new(airp_details: AirportDetailsWindow) -> Self {
        Self { airp_details }
    }
}

impl Default for WidgetDetails {
    fn default() -> Self {
        Self::new(AirportDetailsWindow::default())
    }
}
