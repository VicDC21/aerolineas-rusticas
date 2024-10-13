//! Módulo para grafo de nodos.

use rand::distributions::{Distribution, WeightedIndex};
use rand::thread_rng;
use std::collections::HashSet;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::io::Write;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpStream};
use std::thread::{sleep, Builder, JoinHandle};
use std::time::Duration;

use crate::protocol::aliases::{results::Result, types::Byte};
use crate::protocol::errors::error::Error;
use crate::protocol::traits::Byteable;
use crate::server::actions::opcode::SvAction;
use crate::server::modes::ConnectionMode;
use crate::server::nodes::node::Node;
use crate::server::nodes::states::endpoints::PORT;

/// El ID c on el que comenzar a contar los nodos.
const START_ID: u8 = 10;
/// Cantidad de vecinos a los cuales un nodo tratará de acercarse en un ronda de _gossip_.
const HANDSHAKE_NEIGHBOURS: u8 = 3;
/// La cantidad de nodos que comenzarán su intercambio de _gossip_ con otros [n](crate::server::nodes::graph::HANDSHAKE_NEIGHBOURS) nodos.
const SIMULTANEOUS_GOSSIPERS: u8 = 3;

/// Un grafo es una colección de nodos.
pub struct NodeGraph {
    /// Todos los IDs de nodos bajo este grafo.
    node_ids: Vec<u8>,

    /// Los pesos de los nodos.
    node_weights: Vec<u8>,

    /// El próximo id disponible para un nodo.
    prox_id: u8,

    /// El modo con el que generar los siguientes nodos.
    preferred_mode: ConnectionMode,
}

impl NodeGraph {
    /// Crea un nuevo grafo.
    pub fn new(node_ids: Vec<u8>, prox_id: u8, preferred_mode: ConnectionMode) -> Self {
        Self {
            node_ids,
            prox_id,
            preferred_mode,
            node_weights: Vec::new(),
        }
    }

    /// Crea un nuevo grafo con el modo de conexión preferido.
    pub fn with_mode(preferred_mode: ConnectionMode) -> Self {
        Self::new(Vec::new(), START_ID, preferred_mode)
    }

    /// Genera un vector de los IDs de los nodos.
    pub fn get_ids(&self) -> Vec<u8> {
        self.node_ids.clone()
    }

    /// Genera un vector de los pesos de los nodos.
    pub fn get_weights(&self) -> Vec<u8> {
        self.node_weights.clone()
    }

    /// "Inicia" los nodos del grafo en sus propios hilos.
    pub fn bootup(&mut self, n: u8) -> Result<Vec<JoinHandle<Result<()>>>> {
        self.node_weights = vec![1; n as usize];
        let mut handlers: Vec<JoinHandle<Result<()>>> = Vec::new();
        for _ in 0..n {
            let current_id = self.add_node_id();
            let mut node = Node::new(current_id, self.preferred_mode.clone());

            let builder = Builder::new().name(format!("{}", current_id));
            let spawn_res = builder.spawn(move || node.listen());
            if let Ok(handler) = spawn_res {
                handlers.push(handler);
            }
        }
        // TODO: Idealmente sólo un nodo o un par (los nodos "seed") deberían tener toda la info
        // al principio. Ya el gossip se encargará de poblar el resto.
        for node_id in self.get_ids() {
            self.send_states_to_node(node_id);
        }
        Ok(handlers)
    }

    /// Realiza una ronda de _gossip_.
    pub fn gossip_round(&self) -> Result<JoinHandle<Result<()>>> {
        let builder = Builder::new().name("gossip".to_string());
        let weights = self.get_weights();
        match builder.spawn(move || {
            sleep(Duration::from_millis(200));
            let dist = if let Ok(dist) = WeightedIndex::new(weights) {
                dist
            } else {
                return Ok(());
            };

            let mut rng = thread_rng();
            let mut selected_ids = HashSet::new();
            while selected_ids.len() < SIMULTANEOUS_GOSSIPERS as usize {
                let selected_id = dist.sample(&mut rng);
                selected_ids.insert(selected_id);
            }

            // TODO: Implementar el envío de mensajes de gossip incluyendo los ids seleccionados
            for selected_id in selected_ids {
                if let Err(err) = Self::send_to_node(selected_id as u8, SvAction::Gossip.as_bytes())
                {
                    println!("Ocurrió un error enviando mensaje de gossip:\n\n{}", err);
                }
            }
            Ok(())
        }) {
            Ok(handler) => Ok(handler),
            Err(_) => Err(Error::ServerError(
                "Error procesando la ronda de gossip de los nodos.".to_string(),
            )),
        }
    }

    /// Agrega un nodo al grafo.
    ///
    /// También devuelve el ID del nodo recién agregado.
    pub fn add_node_id(&mut self) -> u8 {
        self.node_ids.push(self.prox_id);
        self.prox_id += 1;
        self.prox_id - 1
    }

    /// Ordena a todos los nodos existentes que envien su endpoint state al nodo con el ID correspondiente.
    fn send_states_to_node(&self, id: u8) {
        for node_id in self.get_ids() {
            if let Err(err) =
                Self::send_to_node(node_id, SvAction::SendEndpointState(id).as_bytes())
            {
                println!(
                    "Ocurrió un error presentando vecinos de un nodo:\n\n{}",
                    err
                );
            }
        }
    }

    /// Avanza a cada segundo el estado de _heartbeat_ de los nodos.
    pub fn beater(&mut self) -> Result<JoinHandle<Result<()>>> {
        let builder = Builder::new().name("beater".to_string());
        let ids = self.get_ids();
        match builder.spawn(move || {
            sleep(Duration::from_secs(1));
            for node_id in ids {
                if Self::send_to_node(node_id, SvAction::Beat.as_bytes()).is_err() {
                    return Err(Error::ServerError(format!(
                        "Error enviado mensaje a nodo {}",
                        node_id
                    )));
                }
            }
            Ok(())
        }) {
            Ok(handler) => Ok(handler),
            Err(_) => Err(Error::ServerError(
                "Error procesando los beats de los nodos.".to_string(),
            )),
        }
    }

    /// Genera una dirección de socket a partir de un ID.
    fn guess_socket(id: u8) -> SocketAddr {
        SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, id), PORT))
    }

    /// Manda un mensaje a un nodo específico.
    pub fn send_to_node(id: u8, bytes: Vec<Byte>) -> Result<()> {
        let addr = Self::guess_socket(id);
        let mut stream = match TcpStream::connect(addr) {
            Ok(tcpstream) => tcpstream,
            Err(_) => {
                return Err(Error::ServerError(format!(
                    "No se pudo conectar al nodo con ID {}",
                    id
                )))
            }
        };
        if stream.write_all(&bytes[..]).is_err() {
            return Err(Error::ServerError(format!(
                "No se pudo escribir el contenido en {}",
                addr
            )));
        }
        Ok(())
    }

    /// Selecciona un ID de nodo conforme al _hashing_ de un conjunto de [Byte]s.
    pub fn select_node(&self, bytes: &Vec<Byte>) -> u8 {
        let mut hasher = DefaultHasher::new();
        bytes.hash(&mut hasher);
        let hash_val = hasher.finish();

        let n = self.node_ids.len() as u64;
        let magic_ind = (hash_val % n) as usize;
        self.node_ids[magic_ind]
    }

    /// Manda un mensaje al nodo relevante mediante el _hashing_ del mensaje.
    pub fn send_message(&self, bytes: Vec<Byte>) -> Result<()> {
        Self::send_to_node(self.select_node(&bytes), bytes)
    }

    /// Apaga todos los nodos.
    pub fn shutdown(&self) {
        for node_id in self.get_ids() {
            if let Err(err) = Self::send_to_node(node_id, SvAction::Exit.as_bytes()) {
                println!("Ocurrió un error saliendo de un nodo:\n\n{}", err);
            }
        }
    }
}

impl Default for NodeGraph {
    fn default() -> Self {
        Self::new(Vec::new(), START_ID, ConnectionMode::Parsing)
    }
}
