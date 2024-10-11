//! Módulo de nodos.

use std::cmp::{Ordering, PartialEq, PartialOrd};

use crate::server::nodes::states::endpoints::EndpointState;

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
    pub fn new(id: u8) -> Self {
        Self {
            id,
            neighbours: Vec::<usize>::new(),
            endpoint_state: EndpointState::with_id(id),
        }
    }

    /// Ve si el nodo es un nodo "hoja".
    pub fn leaf(&self) -> bool {
        self.neighbours.is_empty()
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
