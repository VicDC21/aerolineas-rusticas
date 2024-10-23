//! Módulo para decidir el tipo de puerto.

use std::convert::Into;

/// Los nodos pueden utilizar varios puertos en sus conexiones.
pub enum PortType {
    /// EL puerto para escuchas _requests_ de clientes.
    Cli,

    /// El puerto para comunicaciones internas.
    Priv,
}

impl Into<u16> for PortType {
    fn into(self) -> u16 {
        match self {
            Self::Cli => 8080,
            Self::Priv => 6174,
        }
    }
}
