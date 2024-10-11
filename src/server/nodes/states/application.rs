//! Módulo para el _Application State_ de un nodo.

use crate::server::nodes::states::appstatus::AppStatus;

/// El estado de aplicación contiene otros datos actuales sobre el estado del nodo.
pub struct AppState {
    /// El estado de conexión del nodo.
    status: AppStatus,
}

impl AppState {
    /// Crea una nueva instancia de estado de aplicación.
    pub fn new(status: AppStatus) -> Self {
        Self { status }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new(AppStatus::Bootstrap)
    }
}
