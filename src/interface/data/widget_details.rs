//! Módulo para persisitir las configs de algunos widgets.

use crate::interface::windows::flight_editor::FlightEditorWindow;

/// Detalles de algunos widgets que necesitan persistir entre ciclos.
pub struct WidgetDetails {
    /// Editor de un vuelo.
    pub flight_editor: Option<FlightEditorWindow>,
}

impl WidgetDetails {
    /// Crea una nueva instancia de los detalles de widgets.
    pub fn new(flight_editor: Option<FlightEditorWindow>) -> Self {
        Self { flight_editor }
    }
}

impl Default for WidgetDetails {
    fn default() -> Self {
        Self::new(None)
    }
}
