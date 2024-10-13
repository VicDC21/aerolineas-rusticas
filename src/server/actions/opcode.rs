//! Módulo para una acción especial del servidor.

use std::convert::TryFrom;

use crate::protocol::aliases::types::Byte;
use crate::protocol::errors::error::Error;
use crate::protocol::traits::Byteable;
use crate::server::nodes::states::endpoints::EndpointState;

const ACTION_MASK: u8 = 0xF0;

/// Una "acción" de servidor es un mensaje especial que no entra en ninguna especificaión
/// del protocolo de Cassandra, y en su lugar es usado para acciones especiales fuera
/// del parseo de _queries_.
pub enum SvAction {
    /// Finalizar la conexión actual.
    Exit,

    /// Aumentar en el tiempo los estados de los nodos.
    Beat,

    /// Iniciar ronda de _Gossip_.
    Gossip,

    /// Actualizar el peso de un nodo, segun el id correspondiente.
    SetWeight(u8),

    /// Añadir un nuevo vecino.
    NewNeighbour(EndpointState),

    /// Pedirle a un nodo que envie su endpoint state a otro nodo, dado el ID de este ultimo.
    SendEndpointState(u8),
}

impl SvAction {
    /// Consulta si el conjunto de bytes dados empieza por el prefijo relevante.
    ///
    /// Esto es, si los 4 bits más significativos son todos `1`.
    pub fn is_action(bytes: &[Byte]) -> bool {
        if bytes.is_empty() {
            return false;
        }

        let opcode = bytes[0];
        (opcode & ACTION_MASK) == ACTION_MASK
    }

    /// Potencialmente consigue una acción.
    pub fn get_action(bytes: &[Byte]) -> Option<Self> {
        match Self::try_from(bytes) {
            Ok(action) => Some(action),
            Err(_) => None,
        }
    }
}

impl Byteable for SvAction {
    fn as_bytes(&self) -> Vec<Byte> {
        match self {
            Self::Exit => vec![0xF0],
            Self::Beat => vec![0xF1],
            Self::Gossip => vec![0xF2],
            Self::SetWeight(weight) => vec![0xF3, *weight],
            Self::SendEndpointState(id) => vec![0xF4, *id],
            Self::NewNeighbour(state) => {
                let mut bytes = vec![0xF5];
                bytes.extend(state.as_bytes());
                bytes
            }
        }
    }
}

impl TryFrom<&[Byte]> for SvAction {
    type Error = Error;
    fn try_from(bytes: &[Byte]) -> Result<Self, Self::Error> {
        if bytes.is_empty() {
            return Err(Error::ServerError(
                "Conjunto de bytes demasiado chico.".to_string(),
            ));
        }

        let first = bytes[0];
        if !Self::is_action(bytes) {
            return Err(Error::ServerError(format!(
                "Conjunto de bytes no empieza por `1111...`. En su lugar se recibió {:#b}",
                first
            )));
        }

        match first {
            0xF0 => Ok(Self::Exit),
            0xF1 => Ok(Self::Beat),
            0xF2 => Ok(Self::Gossip),
            0xF3 => {
                if bytes.len() < 2 {
                    return Err(Error::ServerError(
                        "Conjunto de bytes demasiado chico para `SetWeight`.".to_string(),
                    ));
                }
                Ok(Self::SetWeight(bytes[1]))
            }
            0xF4 => {
                if bytes.len() < 2 {
                    return Err(Error::ServerError(
                        "Conjunto de bytes demasiado chico para `NewNeighbour`.".to_string(),
                    ));
                }
                let state = EndpointState::try_from(&bytes[1..])?;
                Ok(Self::NewNeighbour(state))
            }
            0xF5 => {
                if bytes.len() < 2 {
                    return Err(Error::ServerError(
                        "Conjunto de bytes demasiado chico para `SendEndpointState`.".to_string(),
                    ));
                }
                Ok(Self::SendEndpointState(bytes[1]))
            }
            _ => Err(Error::ServerError(format!(
                "'{:#b}' no es un id de acción válida.",
                first
            ))),
        }
    }
}
