//! Módulo para grafo de nodos.

use std::hash::DefaultHasher;
use std::net::TcpStream;

use crate::server::modes::ConnectionMode;
use crate::server::nodes::node::Node;

const START_ID: u8 = 10;

/// Un grafo es una colección de nodos.
pub struct NodeGraph {
    /// Todos los nodos bajo este grafo.
    nodes: Vec<Node>,

    /// El próximo id disponible para un nodo.
    prox_id: u8,

    /// El modo con el que generar los siguientes nodos.
    preferred_mode: ConnectionMode,
}

impl NodeGraph {
    /// Crea un nuevo grafo.
    pub fn new(nodes: Vec<Node>, prox_id: u8, preferred_mode: ConnectionMode) -> Self {
        Self {
            nodes,
            prox_id,
            preferred_mode,
        }
    }

    /// Crea un nuevo grafo con el modo de conexión preferido.
    pub fn with_mode(preferred_mode: ConnectionMode) -> Self {
        Self::new(Vec::new(), START_ID, preferred_mode)
    }

    /// Agrega un nodo al grafo.
    pub fn add_node(&mut self) {
        self.nodes
            .push(Node::new(self.prox_id, self.preferred_mode.clone()));
        self.prox_id += 1;
    }

    // /// Seleciona un nodo en base a un valor de hash.
    // ///
    // /// Esto entra en consistencia con el concepto de [_Consistent Hashing_](https://cassandra.apache.org/doc/latest/cassandra/architecture/dynamo.html#dataset-partitioning-consistent-hashing).
    // pub fn select_node(&self, hash: Hash) -> Node {

    // }

    // /// Deriva una _request_ a un nodo.
    // pub fn derive(&self, stream: TcpStream) -> Result<()> {

    // }
}

impl Default for NodeGraph {
    fn default() -> Self {
        Self::new(Vec::new(), START_ID, ConnectionMode::Parsing)
    }
}
