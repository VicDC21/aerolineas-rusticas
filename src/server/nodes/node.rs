//! Módulo de nodos.

use std::cmp::PartialEq;
use std::collections::{HashMap, HashSet};
use std::io::Read;
use std::net::TcpListener;

use crate::protocol::aliases::{results::Result, types::Byte};
use crate::protocol::errors::error::Error;
use crate::protocol::traits::Byteable;
use crate::server::actions::opcode::{GossipInfo, SvAction};
use crate::server::modes::ConnectionMode;
use crate::server::nodes::states::{
    appstatus::AppStatus,
    endpoints::EndpointState,
    heartbeat::{GenType, VerType},
};
use crate::server::nodes::utils::send_to_node;

/// El ID de un nodo. No se tienen en cuenta casos de cientos de nodos simultáneos,
/// así que un byte debería bastar para representarlo.
pub type NodeId = u8;

/// Mapea todos los estados de los vecinos y de sí mismo.
pub type NodesMap = HashMap<NodeId, EndpointState>;

/// Un nodo es una instancia de parser que se conecta con otros nodos para procesar _queries_.
pub struct Node {
    /// El ID del nodo mismo.
    id: NodeId,

    /// Los estados de los nodos vecinos, incluyendo este mismo.
    ///
    /// No necesariamente serán todos los otros nodos del grafo, sólo los que este nodo conoce.
    neighbours_states: NodesMap,

    /// Estado actual del nodo.
    endpoint_state: EndpointState,
}

impl Node {
    /// Crea un nuevo nodo.
    pub fn new(id: NodeId, mode: ConnectionMode) -> Self {
        Self {
            id,
            neighbours_states: NodesMap::new(),
            endpoint_state: EndpointState::with_id_and_mode(id, mode),
        }
    }

    /// Consulta el ID del nodo.
    pub fn get_id(&self) -> &NodeId {
        &self.id
    }

    /// Consulta el estado del nodo.
    pub fn get_endpoint_state(&self) -> &EndpointState {
        &self.endpoint_state
    }

    /// Compara si el _heartbeat_ de un nodo es más nuevo que otro.
    pub fn is_newer(&self, other: &Self) -> bool {
        self.endpoint_state.is_newer(&other.endpoint_state)
    }

    /// Envia su endpoint state al nodo del ID correspondiente.
    fn send_endpoint_state(&mut self, id: NodeId) {
        if let Err(err) = send_to_node(
            id,
            SvAction::NewNeighbour(self.get_endpoint_state().clone()).as_bytes(),
        ) {
            println!(
                "Ocurrió un error presentando vecinos de un nodo:\n\n{}",
                err
            );
        }
    }

    /// Consulta si ya se tiene un [EndpointState].
    ///
    /// No compara los estados en profundidad, sólo verifica si se tiene un estado
    /// con la misma IP.
    pub fn has_endpoint_state(&self, state: &EndpointState) -> bool {
        self.neighbours_states.contains_key(&state.guess_id())
    }

    fn add_neighbour_state(&mut self, state: EndpointState) {
        if !self.has_endpoint_state(&state) {
            self.neighbours_states.insert(state.guess_id(), state);
        }
    }

    /// Ve si el nodo es un nodo "hoja".
    pub fn leaf(&self) -> bool {
        self.neighbours_states.is_empty()
    }

    /// Consulta el modo de conexión del nodo.
    pub fn mode(&self) -> &ConnectionMode {
        self.endpoint_state.get_appstate().get_mode()
    }

    /// Consulta si el nodo todavía esta booteando.
    pub fn is_bootstraping(&self) -> bool {
        matches!(
            self.endpoint_state.get_appstate().get_status(),
            AppStatus::Bootstrap
        )
    }

    /// Consulta el estado de _heartbeat_.
    pub fn get_beat(&mut self) -> (GenType, VerType) {
        self.endpoint_state.get_heartbeat().as_tuple()
    }

    /// Avanza el tiempo para el nodo.
    pub fn beat(&mut self) -> VerType {
        self.endpoint_state.beat()
    }

    /// Inicia un intercambio de _gossip_ con los vecinos dados.
    pub fn gossip(&mut self, neighbours: HashSet<NodeId>) -> Result<()> {
        // for neighbour in neighbours {
        //     if let Err(err) = send_to_node(neighbour, SvAction::Syn(self.neighbours_states.clone()).as_bytes()) {
        //         println!("Ocurrió un error en medio de una ronda de gossip:\n\n{}", err);
        //     }
        // }
        Ok(())
    }

    /// Se recibe un mensaje [SYN](crate::server::actions::opcode::SvAction::Syn).
    pub fn syn(&mut self, gossip_info: GossipInfo) -> Result<()> {
        Ok(())
    }

    /// Se recibe un mensaje [ACK](crate::server::actions::opcode::SvAction::Ack).
    pub fn ack(&mut self, gossip_info: GossipInfo, nodes_map: NodesMap) -> Result<()> {
        Ok(())
    }

    /// Se recibe un mensaje [ACK2](crate::server::actions::opcode::SvAction::Ack2).
    pub fn ack2(&mut self, nodes_map: NodesMap) -> Result<()> {
        Ok(())
    }

    /// Escucha por los eventos que recibe.
    pub fn listen(&mut self) -> Result<()> {
        let listener = match TcpListener::bind(self.endpoint_state.get_addr()) {
            Ok(tcp_listener) => tcp_listener,
            Err(_) => {
                return Err(Error::ServerError(format!(
                    "No se pudo bindear a la dirección '{}'",
                    self.endpoint_state.get_addr()
                )))
            }
        };

        for tcp_stream_res in listener.incoming() {
            match tcp_stream_res {
                Err(_) => {
                    return Err(Error::ServerError(format!(
                        "Un cliente no pudo conectarse al nodo con ID {}",
                        self.id
                    )))
                }
                Ok(tcp_stream) => {
                    let bytes: Vec<Byte> = tcp_stream.bytes().flatten().collect();
                    match SvAction::get_action(&bytes[..]) {
                        Some(action) => match action {
                            SvAction::Exit => break,
                            SvAction::Beat => {
                                self.beat();
                            }
                            SvAction::Gossip(neighbours) => {
                                self.gossip(neighbours)?;
                            }
                            SvAction::Syn(gossip_info) => {
                                self.syn(gossip_info)?;
                            }
                            SvAction::Ack(gossip_info, nodes_map) => {
                                self.ack(gossip_info, nodes_map)?;
                            }
                            SvAction::Ack2(nodes_map) => {
                                self.ack2(nodes_map)?;
                            }
                            SvAction::NewNeighbour(state) => {
                                self.add_neighbour_state(state);
                            }
                            SvAction::SendEndpointState(id) => {
                                self.send_endpoint_state(id);
                            }
                        },
                        None => {
                            match self.mode() {
                                ConnectionMode::Echo => {
                                    if let Ok(line) = String::from_utf8(bytes) {
                                        println!("[{} - ECHO] {}", self.id, line);
                                    }
                                }
                                ConnectionMode::Parsing => {
                                    // Parsear la query
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.endpoint_state.eq(&other.endpoint_state)
    }
}
