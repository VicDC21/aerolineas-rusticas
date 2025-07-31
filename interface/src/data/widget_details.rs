//! Módulo para persisitir las configs de algunos widgets.

use {crate::windows::flight_editor::FlightEditorWindow, data::login_info::LoginInfo};

/// Detalles de algunos widgets que necesitan persistir entre ciclos.
pub struct WidgetDetails {
    /// Editor de un vuelo.
    pub flight_editor: Option<FlightEditorWindow>,

    /// Información de logueo.
    pub login_info: LoginInfo,

    /// El usuario apretó el botón de 'login' al menos una vez.
    pub has_logged_in: bool,
}

impl WidgetDetails {
    /// Crea una nueva instancia de los detalles de widgets.
    pub fn new(
        flight_editor: Option<FlightEditorWindow>,
        login_info: LoginInfo,
        has_logged_in: bool,
    ) -> Self {
        Self {
            flight_editor,
            login_info,
            has_logged_in,
        }
    }

    /// El usuario se logueó por primera vez.
    pub fn has_logged(&mut self) {
        self.has_logged_in = true;
    }
}

impl Default for WidgetDetails {
    fn default() -> Self {
        Self::new(None, LoginInfo::default(), false)
    }
}
