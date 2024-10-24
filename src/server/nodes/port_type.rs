//! Módulo para decidir el tipo de puerto.

use std::convert::From;

/// Los nodos pueden utilizar varios puertos en sus conexiones.
pub enum PortType {
    /// EL puerto para escuchas _requests_ de clientes.
    Cli,

    /// El puerto para comunicaciones internas.
    Priv,
}

impl PortType {
    /// Transforma el tipo de puerto al número que es.
    pub fn to_num(&self) -> u16 {
        match self {
            Self::Cli => 8080,
            Self::Priv => 6174,
        }
    }
}

impl From<u16> for PortType {
    fn from(value: u16) -> Self {
        match value {
            8080 => Self::Cli,
            6174 => Self::Priv,
            _ => Self::Cli,
        }
    }
}
