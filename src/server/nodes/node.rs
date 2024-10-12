//! Módulo de nodos.

use std::cmp::{Ordering, PartialEq, PartialOrd};
use std::io::{BufRead, BufReader};
use std::net::TcpListener;

use crate::protocol::aliases::results::Result;
use crate::protocol::errors::error::Error;
use crate::server::modes::ConnectionMode;
use crate::server::nodes::states::appstatus::AppStatus;
use crate::server::nodes::states::endpoints::EndpointState;
use crate::server::nodes::states::heartbeat::{GenType, VerType};

/// Un nodo es una instancia de parser que se conecta con otros nodos para procesar _queries_.
pub struct Node {
    /// El ID del nodo mismo.
    id: u8,

    /// Los IDs de los nodos vecinos.
    ///
    /// No necesariamente serán todos los otros nodos del grafo, sólo los que este nodo conoce.
    neighbours: Vec<usize>,

    /// Estado actual del nodo.
    endpoint_state: EndpointState,
}

impl Node {
    /// Crea un nuevo nodo.
    pub fn new(id: u8, mode: ConnectionMode) -> Self {
        Self {
            id,
            neighbours: Vec::<usize>::new(),
            endpoint_state: EndpointState::with_id_and_mode(id, mode),
        }
    }

    /// Consulta el ID del nodo.
    pub fn get_id(&self) -> &u8 {
        &self.id
    }

    /// Ve si el nodo es un nodo "hoja".
    pub fn leaf(&self) -> bool {
        self.neighbours.is_empty()
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
    pub fn listen(&self) -> Result<()> {
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
                    match self.mode() {
                        ConnectionMode::Echo => {
                            let reader = BufReader::new(tcp_stream);
                            let mut lines = reader.lines();
                            while let Some(Ok(line)) = lines.next() {
                                println!("[{} - ECHO] {:?}", self.id, line);
                            }
                        }
                        ConnectionMode::Parsing => {
                            // Parsear la query
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
