//! Módulo para el _Application State_ de un nodo.

use std::convert::TryFrom;

use crate::protocol::{aliases::types::Byte, errors::error::Error, traits::Byteable};
use crate::server::{modes::ConnectionMode, nodes::states::appstatus::AppStatus};

/// El estado de aplicación contiene otros datos actuales sobre el estado del nodo.
#[derive(Debug, Clone)]
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

    /// Establece el estado del nodo.
    pub fn set_status(&mut self, status: AppStatus) {
        self.status = status;
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

impl Byteable for AppState {
    fn as_bytes(&self) -> Vec<Byte> {
        let mut bytes_vec: Vec<Byte> = Vec::new();

        bytes_vec.extend(self.status.as_bytes());
        bytes_vec.extend(self.conmode.as_bytes());

        bytes_vec
    }
}

impl TryFrom<&[Byte]> for AppState {
    type Error = Error;
    fn try_from(bytes: &[Byte]) -> Result<Self, Self::Error> {
        let mut i = 0;

        let status = AppStatus::try_from(&bytes[i..])?;
        i += status.as_bytes().len();

        let conmode = ConnectionMode::try_from(&bytes[i..])?;
        Ok(AppState::new(status, conmode))
    }
}
