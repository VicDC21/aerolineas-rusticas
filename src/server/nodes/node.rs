//! Módulo de nodos.

use std::cmp::{Ordering, PartialEq, PartialOrd};
use std::io::Read;
use std::net::TcpListener;

use crate::protocol::aliases::{results::Result, types::Byte};
use crate::protocol::errors::error::Error;
use crate::protocol::traits::Byteable;
use crate::server::actions::opcode::SvAction;
use crate::server::modes::ConnectionMode;
use crate::server::nodes::states::appstatus::AppStatus;
use crate::server::nodes::states::endpoints::EndpointState;
use crate::server::nodes::states::heartbeat::{GenType, VerType};

use super::graph::NodeGraph;

/// Un nodo es una instancia de parser que se conecta con otros nodos para procesar _queries_.
pub struct Node {
    /// El ID del nodo mismo.
    id: u8,

    /// Los IDs de los nodos vecinos.
    ///
    /// No necesariamente serán todos los otros nodos del grafo, sólo los que este nodo conoce.
    neighbours_states: Vec<EndpointState>,

    /// Estado actual del nodo.
    endpoint_state: EndpointState,
}

impl Node {
    /// Crea un nuevo nodo.
    pub fn new(id: u8, mode: ConnectionMode) -> Self {
        Self {
            id,
            neighbours_states: Vec::<EndpointState>::new(),
            endpoint_state: EndpointState::with_id_and_mode(id, mode),
        }
    }

    /// Consulta el ID del nodo.
    pub fn get_id(&self) -> &u8 {
        &self.id
    }

    /// Consulta el estado del nodo.
    pub fn get_endpoint_state(&self) -> &EndpointState {
        &self.endpoint_state
    }

    /// Envia su endpoint state al nodo del ID correspondiente.
    fn send_endpoint_state(&mut self, id: u8) {
        if let Err(err) = NodeGraph::send_to_node(
            id,
            SvAction::NewNeighbour(self.get_endpoint_state().clone()).as_bytes(),
        ) {
            println!(
                "Ocurrió un error presentando vecinos de un nodo:\n\n{}",
                err
            );
        }
    }

    fn add_neighbour_state(&mut self, state: EndpointState) {
        self.neighbours_states.push(state);
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
                        Some(action) => {
                            match action {
                                SvAction::Exit => break,
                                SvAction::Beat => {
                                    self.beat();
                                }
                                SvAction::Gossip => {
                                    // Implementar ronda de gossip
                                }
                                SvAction::NewNeighbour(state) => {
                                    self.add_neighbour_state(state);
                                }
                                SvAction::SendEndpointState(id) => {
                                    self.send_endpoint_state(id);
                                }
                            }
                        }
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

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.endpoint_state.partial_cmp(&other.endpoint_state)
    }
}
