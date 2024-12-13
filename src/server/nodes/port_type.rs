//! Módulo para decidir el tipo de puerto.

use {crate::protocol::aliases::types::UShort, std::convert::From};

/// Los nodos pueden utilizar varios puertos en sus conexiones.
#[derive(Clone)]
pub enum PortType {
    /// EL puerto para escuchas _requests_ de clientes.
    Cli,

    /// El puerto para comunicaciones internas.
    Priv,
}

impl PortType {
    /// Transforma el tipo de puerto al número que es.
    pub fn to_num(&self) -> UShort {
        match self {
            Self::Cli => 8080,
            Self::Priv => 6174,
        }
    }
}

impl From<UShort> for PortType {
    fn from(value: UShort) -> Self {
        match value {
            8080 => Self::Cli,
            6174 => Self::Priv,
            _ => Self::Cli,
        }
    }
}
