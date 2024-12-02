//! Módulo para persisitir las configs de algunos widgets.

use crate::interface::{data::login_info::LoginInfo, windows::flight_editor::FlightEditorWindow};

/// Detalles de algunos widgets que necesitan persistir entre ciclos.
pub struct WidgetDetails {
    /// Editor de un vuelo.
    pub flight_editor: Option<FlightEditorWindow>,

    /// Información de logueo.
    pub login_info: LoginInfo,
}

impl WidgetDetails {
    /// Crea una nueva instancia de los detalles de widgets.
    pub fn new(flight_editor: Option<FlightEditorWindow>, login_info: LoginInfo) -> Self {
        Self {
            flight_editor,
            login_info,
        }
    }
}

impl Default for WidgetDetails {
    fn default() -> Self {
        Self::new(None, LoginInfo::default())
    }
}
