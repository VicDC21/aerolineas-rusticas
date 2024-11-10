//! Módulo para una acción especial del servidor.

use std::{
    collections::{HashMap, HashSet},
    convert::TryFrom,
};

use crate::protocol::{
    aliases::{
        results::Result as SvResult,
        types::{Byte, Int},
    },
    errors::error::Error,
    traits::Byteable,
    utils::{encode_iter_to_bytes, encode_string_to_bytes, parse_bytes_to_string},
};
use crate::server::nodes::{
    node::{NodeId, NodesMap},
    states::{endpoints::EndpointState, heartbeat::HeartbeatState},
};

/// Un mapa de [EndpointState]s, tal que se puedan pasar entre nodos.
pub type EndpointsVec = Vec<EndpointState>;

/// Contiene los metadatos mínimos para comparar versiones de nodos.
pub type GossipInfo = HashMap<NodeId, HeartbeatState>;

const ACTION_MASK: Byte = 0xF0;

/// Una "acción" de servidor es un mensaje especial que no entra en ninguna especificaión
/// del protocolo de Cassandra, y en su lugar es usado para acciones especiales fuera
/// del parseo de _queries_.
pub enum SvAction {
    /// Finalizar la conexión actual.
    ///
    /// Lo acompaña un [bool] indicando si también terminar las estructuras internas.
    Exit(bool),

    /// Aumentar en el tiempo los estados de los nodos.
    Beat,

    /// Iniciar ronda de _Gossip_.
    ///
    /// Contiene un set de [ID](crate::server::nodes::node::NodeId)s que son los vecinos con los
    /// que este nodo debe interactuar.
    Gossip(HashSet<NodeId>),

    /// Inicia el _handshake_ en un intercambio de _gossip_.
    ///
    /// Acá todavía no se mandan estados de nodo, sino sólo metadatos que son lo mínimo y necesario
    /// para comparar versiones.
    Syn(NodeId, GossipInfo),

    /// Potencial primera respuesta en un intercambio de _gossip_.
    ///
    /// Acá se devuelven tanto los estados que el nodo receptor pide actualizar en un [GossipInfo],
    /// así como los [EndpointState] actualizados que le hacen falta al nodo emisor, en un [NodesMap].
    Ack(NodeId, GossipInfo, NodesMap),

    /// Potencial segunda respuesta en un intercambio de _gossip_.
    ///
    /// A estas alturas sólo se mandan estados que el nodo receptor dijo que le hacían falta,
    /// en un [NodesMap].
    Ack2(NodesMap),

    /// Añadir un nuevo vecino.
    NewNeighbour(EndpointState),

    /// Pedirle a este nodo que envie su endpoint state a otro nodo, dado el ID de este último.
    SendEndpointState(NodeId),

    /// Query enviada internamente por otro nodo.
    InternalQuery(Vec<Byte>),

    /// Guardar metadatos de un nodo en el archivo de metadatos de los nodos `nodes.csv`.
    StoreMetadata,

    /// Obtiene unicamente las filas de la tabla solicitada, pero con sus timestamps.
    GetTableWithTimestampOfRows(String, Vec<Byte>),

    /// Pide unicamente el valor hasheado de una query normal.
    GetResponseHashed(Vec<Byte>),
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

    /// Serializa la información de _gossip_.
    fn encode_gossip_info_to_bytes(gossip_info: &GossipInfo) -> Vec<Byte> {
        let mut bytes_vec: Vec<Byte> = Vec::new();
        let gossip_len_bytes = gossip_info.len().to_le_bytes();
        bytes_vec.extend_from_slice(&[
            gossip_len_bytes[3],
            gossip_len_bytes[2],
            gossip_len_bytes[1],
            gossip_len_bytes[0],
        ]);

        for (node_id, node_heartbeat) in gossip_info {
            bytes_vec.push(node_id.to_owned());
            bytes_vec.extend(node_heartbeat.as_bytes());
        }

        bytes_vec
    }

    /// Serializa un mapa de nodos.
    fn encode_nodes_map_to_bytes(nodes_map: &NodesMap) -> Vec<Byte> {
        let mut bytes_vec: Vec<Byte> = Vec::new();
        let nodes_len_bytes = nodes_map.len().to_le_bytes();
        bytes_vec.extend_from_slice(&[
            nodes_len_bytes[3],
            nodes_len_bytes[2],
            nodes_len_bytes[1],
            nodes_len_bytes[0],
        ]);

        for (node_id, endpoint_state) in nodes_map {
            bytes_vec.push(node_id.to_owned());
            bytes_vec.extend(endpoint_state.as_bytes());
        }

        bytes_vec
    }

    /// Deserializa una secuencia de [Byte]s de vuelta a un [GossipInfo].
    pub fn parse_bytes_to_gossip_info(bytes: &[Byte], i: &mut usize) -> SvResult<GossipInfo> {
        let mut j: usize = 0;
        let mut gossip_info = GossipInfo::new();

        let gossip_len = Int::from_be_bytes([bytes[j], bytes[j + 1], bytes[j + 2], bytes[j + 3]]);
        j += 4;

        for _ in 0..gossip_len {
            let node_id = bytes[j]; // El nodo siempre tendrá un ID de un byte.
            let heartbeat = HeartbeatState::try_from(&bytes[j + 1..])?;
            j += heartbeat.as_bytes().len() + 1;
            gossip_info.insert(node_id, heartbeat);
        }

        *i += j;
        Ok(gossip_info)
    }

    /// Deserializa una secuencia de [Byte]s de vuelta a un [NodesMap].
    pub fn parse_bytes_to_nodes_map(bytes: &[Byte], i: &mut usize) -> SvResult<NodesMap> {
        let mut j: usize = 0;
        let mut nodes_map = NodesMap::new();

        let nodes_len = Int::from_be_bytes([bytes[j], bytes[j + 1], bytes[j + 2], bytes[j + 3]]);
        j += 4;

        for _ in 0..nodes_len {
            let node_id = bytes[j]; // El nodo siempre tendrá un ID de un byte.
            let endpoint_state = EndpointState::try_from(&bytes[j + 1..])?;
            j += endpoint_state.as_bytes().len() + 1;
            nodes_map.insert(node_id, endpoint_state);
        }

        *i += j;
        Ok(nodes_map)
    }
}

impl Byteable for SvAction {
    fn as_bytes(&self) -> Vec<Byte> {
        match self {
            Self::Exit(proc_stop) => vec![0xF0, (if *proc_stop { 0xF1 } else { 0xF0 })],
            Self::Beat => vec![0xF1],
            Self::Gossip(neighbours) => {
                let mut bytes_vec = vec![0xF2];
                let neighbours_iter = neighbours.iter().map(|byte| vec![byte.to_owned()]);
                bytes_vec.extend(encode_iter_to_bytes(neighbours_iter));
                bytes_vec
            }
            Self::Syn(emissor_id, gossip_info) => {
                let mut bytes_vec = vec![0xF3, *emissor_id];
                bytes_vec.extend(Self::encode_gossip_info_to_bytes(gossip_info));
                bytes_vec
            }
            Self::Ack(receptor_id, gossip_info, nodes_map) => {
                let mut bytes_vec = vec![0xF4, *receptor_id];
                bytes_vec.extend(Self::encode_gossip_info_to_bytes(gossip_info));
                bytes_vec.extend(Self::encode_nodes_map_to_bytes(nodes_map));
                bytes_vec
            }
            Self::Ack2(nodes_map) => {
                let mut bytes_vec = vec![0xF5];
                bytes_vec.extend(Self::encode_nodes_map_to_bytes(nodes_map));
                bytes_vec
            }
            Self::NewNeighbour(state) => {
                let mut bytes = vec![0xF6];
                bytes.extend(state.as_bytes());
                bytes
            }
            Self::SendEndpointState(id) => vec![0xF7, *id],
            Self::InternalQuery(query_bytes) => {
                let mut bytes = vec![0xF8];
                bytes.extend(query_bytes);
                bytes
            }
            Self::StoreMetadata => vec![0xF9],
            Self::GetTableWithTimestampOfRows(table_name, query_bytes) => {
                let mut bytes = vec![0xFA];
                bytes.extend(encode_string_to_bytes(table_name));
                bytes.extend(query_bytes);
                bytes
            }
            Self::GetResponseHashed(query_bytes) => {
                let mut bytes = vec![0xFB];
                bytes.extend(query_bytes);
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
            0xF0 => Ok(Self::Exit(bytes[i + 1] != 0x0)),
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
                let emissor_id = bytes[i];
                i += 1;
                Ok(Self::Syn(
                    emissor_id,
                    Self::parse_bytes_to_gossip_info(&bytes[i..], &mut i)?,
                ))
            }
            0xF4 => {
                i += 1;
                let receptor_id = bytes[i];
                i += 1;
                let gossip_info = Self::parse_bytes_to_gossip_info(&bytes[i..], &mut i)?;
                let nodes_map = Self::parse_bytes_to_nodes_map(&bytes[i..], &mut i)?;
                Ok(Self::Ack(receptor_id, gossip_info, nodes_map))
            }
            0xF5 => {
                i += 1;
                Ok(Self::Ack2(Self::parse_bytes_to_nodes_map(
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
            0xF8 => Ok(Self::InternalQuery(bytes[1..].to_vec())),
            0xF9 => Ok(Self::StoreMetadata),
            0xFA => {
                let table_name = parse_bytes_to_string(&bytes[1..], &mut i)?;
                let query_bytes = bytes[table_name.len() + 1 + 2..].to_vec();
                Ok(Self::GetTableWithTimestampOfRows(table_name, query_bytes))
            }
            0xFB => Ok(Self::GetResponseHashed(bytes[1..].to_vec())),
            _ => Err(Error::ServerError(format!(
                "'{:#b}' no es un id de acción válida.",
                first
            ))),
        }
    }
}
