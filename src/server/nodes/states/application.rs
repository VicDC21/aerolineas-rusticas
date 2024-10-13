//! Módulo para el _Application State_ de un nodo.

use crate::server::modes::ConnectionMode;
use crate::server::nodes::states::appstatus::AppStatus;

/// El estado de aplicación contiene otros datos actuales sobre el estado del nodo.
#[derive(Clone)]
pub struct AppState {
    /// El estado de conexión del nodo.
    status: AppStatus,

    /// El tipo de conexión.
    conmode: ConnectionMode,
}

impl AppState {
    /// Crea una nueva instancia de estado de aplicación.
    pub fn new(status: AppStatus, conmode: ConnectionMode) -> Self {
        Self { status, conmode }
    }

    /// Consulta el estado del nodo.
    pub fn get_status(&self) -> &AppStatus {
        &self.status
    }

    /// Consulta el modo de conexión.
    pub fn get_mode(&self) -> &ConnectionMode {
        &self.conmode
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new(AppStatus::Bootstrap, ConnectionMode::Parsing)
    }
}
