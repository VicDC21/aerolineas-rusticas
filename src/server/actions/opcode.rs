//! Módulo para una acción especial del servidor.

use std::collections::HashSet;
use std::convert::TryFrom;

use crate::protocol::aliases::{
    results::Result as SvResult,
    types::{Byte, Int},
};
use crate::protocol::errors::error::Error;
use crate::protocol::traits::Byteable;
use crate::protocol::utils::encode_iter_to_bytes;
use crate::server::nodes::node::NodeId;
use crate::server::nodes::states::endpoints::EndpointState;

/// Una colección de [EndpointState]s, tal que se puedan pasar entre nodos.
pub type EndpointsVec = Vec<EndpointState>;

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
    ///
    /// Contiene un set de [ID](crate::server::nodes::node::NodeId)s que son los vecinos con los
    /// que este nodo debe interactuar.
    Gossip(HashSet<NodeId>),

    /// Inicia el _handshake_ en un intercambio de _gossip_.
    Syn(EndpointsVec),

    /// Potencial primera respuesta en un intercambio de _gossip_.
    Ack(EndpointsVec),

    /// Potencial segunda respuesta en un intercambio de _gossip_.
    Ack2(EndpointsVec),

    /// Añadir un nuevo vecino.
    NewNeighbour(EndpointState),

    /// Pedirle a este nodo que envie su endpoint state a otro nodo, dado el ID de este último.
    SendEndpointState(NodeId),
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

    /// Potencialmente consigue una acción, ignorando un error de ser el caso.
    pub fn get_action(bytes: &[Byte]) -> Option<Self> {
        match Self::try_from(bytes) {
            Ok(action) => Some(action),
            Err(_) => None,
        }
    }

    /// Serializa una secuencia de [EndpointState]s.
    pub fn encode_endpoint_vec_to_bytes(endpoints: &EndpointsVec) -> Vec<Byte> {
        let mut bytes_vec: Vec<Byte> = Vec::new();
        let endpoints_len_bytes = endpoints.len().to_le_bytes();

        bytes_vec.extend_from_slice(&[
            endpoints_len_bytes[3],
            endpoints_len_bytes[2],
            endpoints_len_bytes[1],
            endpoints_len_bytes[0],
        ]);

        for endpoint_state in endpoints {
            bytes_vec.extend(endpoint_state.as_bytes());
        }

        bytes_vec
    }

    /// Deserializa una secuencia de bytes de vuelta a un [HashSet] de [EndpointState]s.
    pub fn parse_bytes_to_endpoint_vec(bytes: &[Byte], i: &mut usize) -> SvResult<EndpointsVec> {
        let mut j: usize = 0;
        let mut endpoints: EndpointsVec = EndpointsVec::new();
        let endpoints_len =
            Int::from_be_bytes([bytes[j], bytes[j + 1], bytes[j + 2], bytes[j + 3]]);
        j += 4;

        for _ in 0..endpoints_len {
            let endpoint = EndpointState::try_from(&bytes[j..])?;
            j += endpoint.as_bytes().len();
            endpoints.push(endpoint);
        }

        *i += j;
        Ok(endpoints)
    }
}

impl Byteable for SvAction {
    fn as_bytes(&self) -> Vec<Byte> {
        match self {
            Self::Exit => vec![0xF0],
            Self::Beat => vec![0xF1],
            Self::Gossip(neighbours) => {
                let mut bytes_vec = vec![0xF2];
                let neighbours_iter = neighbours.iter().map(|byte| vec![byte.to_owned()]);
                bytes_vec.extend(encode_iter_to_bytes(neighbours_iter));
                bytes_vec
            }
            Self::Syn(endpoints) => {
                let mut bytes_vec = vec![0xF3];
                bytes_vec.extend(Self::encode_endpoint_vec_to_bytes(endpoints));
                bytes_vec
            }
            Self::Ack(endpoints) => {
                let mut bytes_vec = vec![0xF4];
                bytes_vec.extend(Self::encode_endpoint_vec_to_bytes(endpoints));
                bytes_vec
            }
            Self::Ack2(endpoints) => {
                let mut bytes_vec = vec![0xF5];
                bytes_vec.extend(Self::encode_endpoint_vec_to_bytes(endpoints));
                bytes_vec
            }
            Self::SendEndpointState(id) => vec![0xF6, *id],
            Self::NewNeighbour(state) => {
                let mut bytes = vec![0xF7];
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

        let mut i = 0;
        let first = bytes[i];
        if !Self::is_action(bytes) {
            return Err(Error::ServerError(format!(
                "Conjunto de bytes no empieza por `1111...`. En su lugar se recibió {:#b}",
                first
            )));
        }

        match first {
            0xF0 => Ok(Self::Exit),
            0xF1 => Ok(Self::Beat),
            0xF2 => {
                i += 1;
                let mut set_bytes: HashSet<Byte> = HashSet::new();
                let set_len =
                    Int::from_be_bytes([bytes[i], bytes[i + 1], bytes[i + 2], bytes[i + 3]]);
                i += 4;
                for _ in 0..set_len {
                    let byte_len =
                        Int::from_be_bytes([bytes[i], bytes[i + 1], bytes[i + 2], bytes[i + 3]]);
                    i += 4;
                    if byte_len != 1 {
                        return Err(Error::ServerError(format!(
                            "Se esperaba que el valor del set ocupara sólo un byte, no {}",
                            byte_len
                        )));
                    }

                    set_bytes.insert(bytes[i]);
                    i += 1;
                }
                Ok(Self::Gossip(set_bytes))
            }
            0xF3 => {
                i += 1;
                Ok(Self::Syn(Self::parse_bytes_to_endpoint_vec(
                    &bytes[i..],
                    &mut i,
                )?))
            }
            0xF4 => {
                i += 1;
                Ok(Self::Ack(Self::parse_bytes_to_endpoint_vec(
                    &bytes[i..],
                    &mut i,
                )?))
            }
            0xF5 => {
                i += 1;
                Ok(Self::Ack2(Self::parse_bytes_to_endpoint_vec(
                    &bytes[i..],
                    &mut i,
                )?))
            }
            0xF6 => {
                if bytes.len() < 2 {
                    return Err(Error::ServerError(
                        "Conjunto de bytes demasiado chico para `NewNeighbour`.".to_string(),
                    ));
                }
                let state = EndpointState::try_from(&bytes[1..])?;
                Ok(Self::NewNeighbour(state))
            }
            0xF7 => {
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
