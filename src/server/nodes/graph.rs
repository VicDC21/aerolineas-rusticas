//! Módulo para grafo de nodos.

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

const START_ID: u8 = 10;

/// Un grafo es una colección de nodos.
pub struct NodeGraph {
    /// Todos los IDs de nodos bajo este grafo.
    node_ids: Vec<u8>,

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

    /// "Inicia" los nodos del grafo en sus propios hilos.
    pub fn bootup(&mut self, n: u8) -> Result<Vec<JoinHandle<Result<()>>>> {
        let mut handlers: Vec<JoinHandle<Result<()>>> = Vec::new();
        for _ in 0..n {
            let current_id = self.add_node_id();
            let node = Node::new(current_id, self.preferred_mode.clone());

            let builder = Builder::new().name(format!("{}", current_id));
            let spawn_res = builder.spawn(move || node.listen());
            if let Ok(handler) = spawn_res {
                handlers.push(handler);
            }
        }
        Ok(handlers)
    }

    /// Agrega un nodo al grafo.
    /// 
    /// También devuelve el ID del nodo recién agregado.
    pub fn add_node_id(&mut self) -> u8 {
        self.node_ids.push(self.prox_id);
        self.prox_id += 1;
        self.prox_id - 1
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
    pub fn guess_socket(id: u8) -> SocketAddr {
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

    /// Seleciona un ID de nodo conforme al _hashing_ de un conjunto de [Byte]s.
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
