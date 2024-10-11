//! Módulo para grafo de nodos.

use crate::server::nodes::node::Node;

/// Un grafo es una colección de nodos.
pub struct Graph {
    /// Todos los nodos bajo este grafo.
    nodes: Vec<Node>,

    /// El próximo id disponible para un nodo.
    prox_id: u8,
}

impl Graph {
    /// Crea un nuevo grafo.
    pub fn new(nodes: Vec<Node>, prox_id: u8) -> Self {
        Self { nodes, prox_id }
    }

    /// Agrega un nodo al grafo.
    pub fn add_node(&mut self) {
        self.nodes.push(Node::new(self.prox_id));
        self.prox_id += 1;
    }
}

impl Default for Graph {
    fn default() -> Self {
        Self::new(Vec::new(), 1)
    }
}
