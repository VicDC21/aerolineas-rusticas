//! Paquete para funciones públicas comunes entre tests de integración.
//!
//! Dichas funciones se definen directamente en este archivo, o sino corremos el riesgo
//! de que cargo crea que los archivos son archivos de tests en sí.

use std::thread::{spawn, JoinHandle};

use aerolineas_rusticas::{protocol::aliases::results::Result, server::nodes::graph::NodesGraph};

/// Un handle común en nuestra librería.
pub type ThreadHandle<T> = JoinHandle<Result<T>>;

/// Crea un [grafo](NodesGraph) en modo de DEBUG y lo corre en un hilo aparte.
pub fn init_graph_echo() -> ThreadHandle<()> {
    let mut echo_graph = NodesGraph::echo_mode();

    spawn(move || echo_graph.init())
}
